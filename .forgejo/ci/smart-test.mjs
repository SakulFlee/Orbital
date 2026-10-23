#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT = process.cwd();
const MARKER = '<!-- orbital-ci-test-results -->';
const REPORT_DIR = path.join(ROOT, 'target', 'ci');
const LOG_PATH = path.join(REPORT_DIR, 'test-output.log');
const JUNIT_PATH = path.join(ROOT, 'target', 'nextest', 'ci', 'junit.xml');
const API_URL = (process.env.GITHUB_API_URL || 'https://forgejo.sakul-flee.de/api/v1').replace(/\/+$/, '');
const REPOSITORY = process.env.GITHUB_REPOSITORY || '';
const TOKEN = process.env.FORGEJO_TOKEN || process.env.GITHUB_TOKEN || '';

const FULL_PATHS = new Set([
  'Cargo.toml',
  'Cargo.lock',
  'flake.nix',
  'flake.lock',
  'shell.nix',
  'rust-toolchain',
  'rust-toolchain.toml',
  'renovate.json',
  '.envrc',
  'Orbital.toml',
]);

const FULL_PREFIXES = [
  '.cargo/',
  '.config/',
  '.forgejo/',
  '.github/',
  'Assets/',
  'wit/',
  'CTestMod/',
  'target/',
];

const PUBLIC_SERVER_URL = process.env.PUBLIC_SERVER_URL || 'https://forgejo.sakul-flee.de';

function parseArgs(argv) {
  const options = {
    dryRun: false,
    forceFull: false,
    changedFilesFile: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];

    if (argument === '--dry-run') {
      options.dryRun = true;
      continue;
    }

    if (argument === '--force-full') {
      options.forceFull = true;
      continue;
    }

    if (argument === '--changed-files-file') {
      options.changedFilesFile = argv[index + 1] || null;
      index += 1;
      continue;
    }

    if (argument === '--help' || argument === '-h') {
      console.log('Usage: node .forgejo/ci/smart-test.mjs [--dry-run] [--force-full] [--changed-files-file FILE]');
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${argument}`);
  }

  return options;
}

function readEvent() {
  const eventPath = process.env.GITHUB_EVENT_PATH;

  if (!eventPath || !fs.existsSync(eventPath)) {
    return {};
  }

  try {
    return JSON.parse(fs.readFileSync(eventPath, 'utf8'));
  } catch {
    return {};
  }
}

function readChangedFiles(file) {
  return fs
    .readFileSync(file, 'utf8')
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function isDocumentation(file) {
  if (/\.(md|mdx)$/i.test(file)) {
    return true;
  }

  if (/^LICENSE(\.|$)/i.test(file)) {
    return true;
  }

  if (file.startsWith('Images/') || file.startsWith('.github/logo/')) {
    return true;
  }

  return ['.gitignore', '.gitattributes', '.editorconfig', '.opencodeignore'].includes(file);
}

function loadWorkspace() {
  const result = spawnSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: ROOT,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });

  if (result.status !== 0) {
    throw new Error(result.stderr || 'cargo metadata failed');
  }

  const metadata = JSON.parse(result.stdout);
  const packages = metadata.packages.map((entry) => {
    const dir = path.dirname(entry.manifest_path);

    return {
      name: entry.name,
      dirRel: path.relative(ROOT, dir).split(path.sep).join('/'),
    };
  });

  packages.sort((left, right) => right.dirRel.length - left.dirRel.length);

  return packages;
}

function packageForFile(packages, file) {
  return packages.find((entry) => file === entry.dirRel || file.startsWith(`${entry.dirRel}/`)) || null;
}

function selectTests({ forceFull, changedFiles, event, packages }) {
  const baseRef = event.pull_request?.base?.ref || process.env.GITHUB_BASE_REF || '';
  const prNumber = event.pull_request?.number;

  if (forceFull) {
    return { mode: 'full', reason: 'full test run requested' };
  }

  if (prNumber && baseRef.startsWith('release/')) {
    return { mode: 'full', reason: `pull request targets ${baseRef}` };
  }

  if (!prNumber && changedFiles == null) {
    return { mode: 'full', reason: 'not a pull request' };
  }

  if (changedFiles == null) {
    return { mode: 'full', reason: 'could not determine changed files' };
  }

  const effectiveFiles = changedFiles.filter((file) => file && !isDocumentation(file));

  if (effectiveFiles.length === 0) {
    return { mode: 'skip', reason: 'documentation-only changes' };
  }

  const crates = new Set();
  let examplesOnly = true;

  for (const file of effectiveFiles) {
    if (FULL_PATHS.has(file) || FULL_PREFIXES.some((prefix) => file.startsWith(prefix))) {
      return { mode: 'full', reason: `workspace-level change: ${file}` };
    }

    const entry = packageForFile(packages, file);

    if (!entry) {
      if (file.startsWith('Examples/')) {
        continue;
      }

      return { mode: 'full', reason: `unmapped change: ${file}` };
    }

    if (entry.dirRel.startsWith('Crates/')) {
      crates.add(entry.name);
      examplesOnly = false;
      continue;
    }

    if (entry.dirRel.startsWith('Examples/')) {
      continue;
    }

    return { mode: 'full', reason: `unmapped change: ${file}` };
  }

  if (crates.size === 0) {
    return {
      mode: 'skip',
      reason: examplesOnly ? 'examples-only changes' : 'no testable crates changed',
    };
  }

  return {
    mode: 'scoped',
    reason: 'directly changed crates',
    packages: [...crates].sort(),
  };
}

async function apiRequest(endpoint, options = {}) {
  const headers = {
    Accept: 'application/json',
    ...(options.headers || {}),
  };

  if (TOKEN) {
    headers.Authorization = `token ${TOKEN}`;
  }

  const response = await fetch(`${API_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const body = await response.text();

    throw new Error(`${options.method || 'GET'} ${endpoint} returned ${response.status}: ${body.slice(0, 300)}`);
  }

  return response;
}

async function fetchPullFiles(prNumber) {
  const files = [];
  const limit = 50;

  for (let page = 1; page <= 100; page += 1) {
    const response = await apiRequest(
      `/repos/${REPOSITORY}/pulls/${prNumber}/files?limit=${limit}&page=${page}`,
    );
    const batch = await response.json();

    if (!Array.isArray(batch) || batch.length === 0) {
      break;
    }

    for (const entry of batch) {
      if (entry.filename) {
        files.push(entry.filename);
      }

      if (entry.previous_filename) {
        files.push(entry.previous_filename);
      }
    }

    if (batch.length < limit) {
      break;
    }
  }

  return [...new Set(files)];
}

function decodeXml(value) {
  return value
    .replace(/&#x([0-9a-f]+);/gi, (_, hex) => String.fromCodePoint(Number.parseInt(hex, 16)))
    .replace(/&#(\d+);/g, (_, decimal) => String.fromCodePoint(Number.parseInt(decimal, 10)))
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&apos;/g, "'")
    .replace(/&amp;/g, '&');
}

function parseAttributes(value) {
  const attributes = {};
  const pattern = /([\w:-]+)\s*=\s*"([^"]*)"/g;
  let match = pattern.exec(value);

  while (match) {
    attributes[match[1]] = decodeXml(match[2]);
    match = pattern.exec(value);
  }

  return attributes;
}

function parseJUnit(xml) {
  const cases = [];

  if (!xml) {
    return cases;
  }

  const suites = /<testsuite\b([^>]*)>([\s\S]*?)<\/testsuite>/g;
  let suiteMatch = suites.exec(xml);

  while (suiteMatch) {
    const suiteAttributes = parseAttributes(suiteMatch[1]);
    const suiteName = suiteAttributes.name || 'test suite';
    const packageName = suiteName.split('::')[0] || suiteName;
    const body = suiteMatch[2];
    const testCases = /<testcase\b([^>]*?)(?:\/>|>([\s\S]*?)<\/testcase>)/g;
    let caseMatch = testCases.exec(body);

    while (caseMatch) {
      const attributes = parseAttributes(caseMatch[1]);
      const inner = caseMatch[2] || '';
      const failed = /<failure\b|<error\b/.test(inner);
      const skipped = /<skipped\b/.test(inner);
      const failureMatch = inner.match(/<(failure|error)\b[^>]*>([\s\S]*?)<\/\1>/);

      cases.push({
        packageName,
        suiteName,
        name: attributes.name || '(unnamed)',
        time: Number(attributes.time) || 0,
        status: failed ? 'failed' : skipped ? 'skipped' : 'passed',
        message: failureMatch ? decodeXml(failureMatch[2]).trim() : '',
      });

      caseMatch = testCases.exec(body);
    }

    suiteMatch = suites.exec(xml);
  }

  return cases;
}

function parseDoctests(logText) {
  const packages = new Map();
  let current = null;

  for (const line of logText.split(/\r?\n/)) {
    const documentMatch = line.match(/^\s*Doc-tests\s+(\S+)\s*$/);

    if (documentMatch) {
      current = documentMatch[1];

      if (!packages.has(current)) {
        packages.set(current, {
          passed: 0,
          failed: 0,
          skipped: 0,
          tests: 0,
          failureNames: [],
        });
      }

      continue;
    }

    const resultMatch = line.match(/^test result:\s+(ok|FAILED)\.\s+(\d+) passed;\s+(\d+) failed;\s+(\d+) ignored/);

    if (resultMatch && current) {
      const entry = packages.get(current);
      entry.passed += Number(resultMatch[2]);
      entry.failed += Number(resultMatch[3]);
      entry.skipped += Number(resultMatch[4]);
      entry.tests += Number(resultMatch[2]) + Number(resultMatch[3]) + Number(resultMatch[4]);
      current = null;
      continue;
    }

    const failureMatch = line.match(/^test (.+?) \.\.\. FAILED$/);

    if (failureMatch && current) {
      packages.get(current).failureNames.push(failureMatch[1]);
    }
  }

  return packages;
}

function parseDoctestOutputs(logText) {
  const outputs = new Map();
  let current = null;
  let buffer = [];

  const commit = () => {
    if (current) {
      outputs.set(current, buffer.join('\n').trim());
    }

    current = null;
    buffer = [];
  };

  for (const line of logText.split(/\r?\n/)) {
    const headerMatch = line.match(/^---- (.+) stdout ----$/);

    if (headerMatch) {
      commit();
      current = headerMatch[1].trim();
      continue;
    }

    if (current) {
      if (line === 'failures:' || line.startsWith('test result:')) {
        commit();
        continue;
      }

      buffer.push(line);
    }
  }

  commit();

  return outputs;
}

function aggregate(cases) {
  const packages = new Map();
  const totals = { tests: 0, passed: 0, failed: 0, skipped: 0, time: 0 };

  for (const entry of cases) {
    const current = packages.get(entry.packageName) || {
      tests: 0,
      passed: 0,
      failed: 0,
      skipped: 0,
      time: 0,
      failures: [],
    };

    current.tests += 1;
    current.time += entry.time;
    totals.tests += 1;
    totals.time += entry.time;

    if (entry.status === 'failed') {
      current.failed += 1;
      current.failures.push(entry);
      totals.failed += 1;
    } else if (entry.status === 'skipped') {
      current.skipped += 1;
      totals.skipped += 1;
    } else {
      current.passed += 1;
      totals.passed += 1;
    }

    packages.set(entry.packageName, current);
  }

  return { packages, totals };
}

function formatNumber(value, digits = 2) {
  return Number(value).toFixed(digits);
}

function formatDuration(milliseconds) {
  const seconds = milliseconds / 1000;

  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }

  const minutes = Math.floor(seconds / 60);
  const remainder = Math.round(seconds % 60);

  return `${minutes}m ${remainder}s`;
}

function markdownCell(value) {
  return String(value).replace(/\|/g, '\\|').replace(/\r?\n/g, ' ');
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function mermaidLabel(value, limit = 70) {
  return String(value)
    .replace(/\\/g, '/')
    .replace(/"/g, "'")
    .replace(/[\r\n]+/g, ' ')
    .slice(0, limit);
}

function failureGraph(failures) {
  if (failures.length === 0) {
    return '';
  }

  const visible = failures.slice(0, 20);
  const lines = ['flowchart LR'];
  const packageNodes = new Map();
  const suiteNodes = new Map();
  const nodeNames = new Map();
  let nextId = 0;

  const node = (key, label) => {
    if (!nodeNames.has(key)) {
      const id = `n${nextId}`;
      nextId += 1;
      nodeNames.set(key, id);
      lines.push(`  ${id}["${mermaidLabel(label)}"]:::fail`);
    }

    return nodeNames.get(key);
  };

  for (const failure of visible) {
    const packageKey = `package:${failure.package}`;
    const suiteKey = `suite:${failure.suite}`;
    const packageNode = node(packageKey, failure.package);
    const suiteNode = node(suiteKey, failure.suite);
    const testNode = node(`test:${failure.package}:${failure.suite}:${failure.name}`, failure.name);
    lines.push(`  ${packageNode} --> ${suiteNode} --> ${testNode}`);
  }

  lines.push('  classDef fail fill:#d73a49,color:#ffffff,stroke:#ffffff');

  return lines.join('\n');
}

function buildMarkdown({
  selection,
  cases,
  doctests,
  doctestOutputs = new Map(),
  nextestCode,
  doctestCode,
  elapsedMs,
  nextestLog = '',
  doctestLog = '',
}) {
  const { packages, totals } = aggregate(cases);
  const nextestCommandFailed = nextestCode !== 0;
  const doctestCommandFailed = doctestCode !== 0;
  const failedRun = nextestCommandFailed || doctestCommandFailed;
  const verdict = failedRun ? 'FAIL' : 'PASS';
  const runNumber = process.env.GITHUB_RUN_NUMBER;
  const runUrl = runNumber && REPOSITORY ? `${PUBLIC_SERVER_URL}/${REPOSITORY}/actions/runs/${runNumber}` : null;
  const lines = [];

  lines.push(MARKER);
  lines.push('## Smart test results');
  lines.push('');
  lines.push(`**Result:** ${verdict}`);
  lines.push('');

  if (failedRun) {
    const failingCommands = [
      nextestCommandFailed ? 'nextest' : null,
      doctestCommandFailed ? 'doctest' : null,
    ].filter(Boolean);

    lines.push(`**Failure:** ${failingCommands.join(' and ')} command failed`);
    lines.push('');
  }

  lines.push(`**Scope:** ${selection.mode === 'full' ? 'full workspace' : selection.packages.join(', ')}`);
  lines.push('');
  lines.push(`**Reason:** ${selection.reason}`);
  lines.push('');
  lines.push(`**Elapsed:** ${formatDuration(elapsedMs)}`);
  lines.push('');

  if (runUrl) {
    lines.push(`[Workflow run](${runUrl})`);
    lines.push('');
  }

  let doctestTests = 0;
  let doctestPassed = 0;
  let doctestFailed = 0;
  let doctestSkipped = 0;

  for (const entry of doctests.values()) {
    doctestTests += entry.tests;
    doctestPassed += entry.passed;
    doctestFailed += entry.failed;
    doctestSkipped += entry.skipped;
  }

  const overallTests = totals.tests + doctestTests;
  const overallPassed = totals.passed + doctestPassed;
  const overallFailed = totals.failed + doctestFailed;
  const overallSkipped = totals.skipped + doctestSkipped;

  lines.push(
    `**Overall:** ${overallTests} tests, ${overallPassed} passed, ${overallFailed} failed, ${overallSkipped} skipped`,
  );
  lines.push('');

  if (totals.tests > 0) {
    lines.push(
      `**Nextest:** ${totals.tests} tests, ${totals.passed} passed, ${totals.failed} failed, ${totals.skipped} skipped`,
    );
  } else if (nextestCommandFailed) {
    lines.push('**Nextest:** build failed (no test report)');
  } else {
    lines.push('**Nextest:** no tests reported');
  }

  if (doctestTests > 0) {
    lines.push(
      `**Doctests:** ${doctestTests} tests, ${doctestPassed} passed, ${doctestFailed} failed, ${doctestSkipped} skipped`,
    );
  } else if (doctestCommandFailed) {
    lines.push('**Doctests:** command failed (no test report)');
  } else {
    lines.push('**Doctests:** no tests reported');
  }

  lines.push('');

  if (doctestTests > 0) {
    lines.push('_Outcome charts are split between nextest and doctests. The timing chart covers nextest only._');
    lines.push('');
  }

  const packageNames = [...new Set([...packages.keys(), ...doctests.keys()])]
    .sort()
    .filter((name) => {
      const nextestEntry = packages.get(name);
      const doctestEntry = doctests.get(name);

      return (nextestEntry && nextestEntry.tests > 0) || (doctestEntry && doctestEntry.tests > 0);
    });

  if (packageNames.length > 0) {
    lines.push('| Crate | Nextest | Passed | Failed | Skipped | Doctests | Time |');
    lines.push('| --- | ---: | ---: | ---: | ---: | ---: | ---: |');

    for (const name of packageNames) {
      const nextestEntry = packages.get(name) || {
        tests: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        time: 0,
      };
      const doctestEntry = doctests.get(name);
      const doctestCell = doctestEntry && doctestEntry.tests > 0
        ? `${doctestEntry.passed}/${doctestEntry.tests}${doctestEntry.failed > 0 ? ` (${doctestEntry.failed} failed)` : ''}`
        : '-';

      lines.push(
        `| ${markdownCell(name)} | ${nextestEntry.tests} | ${nextestEntry.passed} | ${nextestEntry.failed} | ${nextestEntry.skipped} | ${doctestCell} | ${formatNumber(nextestEntry.time)}s |`,
      );
    }

    lines.push('');
  }

  const slices = [
    ['Passed', totals.passed],
    ['Failed', totals.failed],
    ['Skipped', totals.skipped],
  ].filter(([, value]) => value > 0);

  if (slices.length > 0) {
    lines.push('```mermaid');
    lines.push('pie showData');
    lines.push('    title Nextest outcomes');

    for (const [label, value] of slices) {
      lines.push(`    "${label}" : ${value}`);
    }

    lines.push('```');
    lines.push('');
  }

  const doctestSlices = [
    ['Passed', doctestPassed],
    ['Failed', doctestFailed],
    ['Skipped', doctestSkipped],
  ].filter(([, value]) => value > 0);

  if (doctestSlices.length > 0) {
    lines.push('```mermaid');
    lines.push('pie showData');
    lines.push('    title Doctest outcomes');

    for (const [label, value] of doctestSlices) {
      lines.push(`    "${label}" : ${value}`);
    }

    lines.push('```');
    lines.push('');
  }

  const slowPackages = [...packages.entries()]
    .filter(([, entry]) => entry.time > 0)
    .sort((left, right) => right[1].time - left[1].time)
    .slice(0, 8);

  if (slowPackages.length >= 2) {
    const maximum = Math.max(...slowPackages.map(([, entry]) => entry.time));
    const axisMaximum = Math.max(1, Math.ceil(maximum * 10) / 10);
    const labels = slowPackages.map(([name]) => `"${mermaidLabel(name, 40)}"`).join(', ');
    const values = slowPackages.map(([, entry]) => formatNumber(entry.time)).join(', ');

    lines.push('```mermaid');
    lines.push('xychart-beta');
    lines.push('    title "Slowest crates (nextest seconds)"');
    lines.push(`    x-axis [${labels}]`);
    lines.push(`    y-axis "Seconds" 0 --> ${axisMaximum}`);
    lines.push(`    bar [${values}]`);
    lines.push('```');
    lines.push('');
  }

  const failures = [];

  for (const [packageName, entry] of packages.entries()) {
    for (const failure of entry.failures) {
      failures.push({
        package: packageName,
        suite: failure.suiteName,
        name: failure.name,
        message: failure.message,
      });
    }
  }

  for (const [packageName, entry] of doctests.entries()) {
    for (const name of entry.failureNames) {
      failures.push({
        package: packageName,
        suite: `${packageName} (doctests)`,
        name,
        message: doctestOutputs.get(name) || '',
      });
    }
  }

  if (failures.length > 0) {
    lines.push('### Failed tests');
    lines.push('');
    lines.push('```mermaid');
    lines.push(failureGraph(failures));
    lines.push('```');
    lines.push('');

    if (failures.length > 20) {
      lines.push(`Only the first 20 of ${failures.length} failed tests are shown in the graph.`);
      lines.push('');
    }

    lines.push('<details>');
    lines.push('<summary>Failure details</summary>');
    lines.push('');

    for (const failure of failures.slice(0, 10)) {
      lines.push(`#### ${escapeHtml(failure.package)}::${escapeHtml(failure.name)}`);
      lines.push('');
      lines.push(`<pre><code>${escapeHtml((failure.message || '(no output captured)').slice(0, 2500))}</code></pre>`);
      lines.push('');
    }

    if (failures.length > 10) {
      lines.push(`_${failures.length - 10} more failures omitted from this comment._`);
      lines.push('');
    }

    lines.push('</details>');
    lines.push('');
  }

  if (nextestCommandFailed && totals.failed === 0) {
    lines.push('<details>');
    lines.push('<summary>Nextest output tail</summary>');
    lines.push('');
    lines.push(`<pre><code>${escapeHtml(tailLines(nextestLog))}</code></pre>`);
    lines.push('');
    lines.push('</details>');
    lines.push('');
  }

  if (doctestCommandFailed && doctestFailed === 0) {
    lines.push('<details>');
    lines.push('<summary>Doctest output tail</summary>');
    lines.push('');
    lines.push(`<pre><code>${escapeHtml(tailLines(doctestLog))}</code></pre>`);
    lines.push('');
    lines.push('</details>');
    lines.push('');
  }

  return lines.join('\n');
}

function tailLines(text, count = 120) {
  return text.split(/\r?\n/).slice(-count).join('\n');
}

function writeStepSummary(markdown) {
  const summaryPath = process.env.GITHUB_STEP_SUMMARY;

  if (!summaryPath) {
    return;
  }

  try {
    fs.appendFileSync(summaryPath, `${markdown}\n`);
  } catch {
    return;
  }
}

async function upsertComment(markdown, prNumber) {
  if (!TOKEN || !REPOSITORY || !prNumber) {
    console.log('[smart-test] Skipping PR comment: missing token, repository, or pull request number');
    return;
  }

  let existing = null;

  for (let page = 1; page <= 100; page += 1) {
    const response = await apiRequest(
      `/repos/${REPOSITORY}/issues/${prNumber}/comments?limit=50&page=${page}`,
    );
    const comments = await response.json();

    if (!Array.isArray(comments) || comments.length === 0) {
      break;
    }

    existing = comments.find((comment) => comment.body?.includes(MARKER)) || null;

    if (existing || comments.length < 50) {
      break;
    }
  }

  const body = JSON.stringify({ body: markdown });

  if (existing) {
    await apiRequest(`/repos/${REPOSITORY}/issues/comments/${existing.id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body,
    });
    return;
  }

  await apiRequest(`/repos/${REPOSITORY}/issues/${prNumber}/comments`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body,
  });
}

function runStreaming(command, args, append) {
  return new Promise((resolve) => {
    const commandLine = `\n$ ${command} ${args.join(' ')}\n`;
    append(commandLine);

    const child = spawn(command, args, {
      cwd: ROOT,
      env: process.env,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let output = '';
    let settled = false;

    const finish = (code) => {
      if (settled) {
        return;
      }

      settled = true;
      resolve({ code, output });
    };

    const consume = (chunk) => {
      const text = chunk.toString();
      output += text;
      append(text);
    };

    child.stdout.on('data', consume);
    child.stderr.on('data', consume);

    child.on('error', (error) => {
      const text = `\nFailed to start ${command}: ${error.message}\n`;
      output += text;
      append(text);
      finish(1);
    });

    child.on('close', (code) => finish(code ?? 1));
  });
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const event = readEvent();
  const prNumber = event.pull_request?.number;
  const packages = loadWorkspace();
  let changedFiles = null;

  if (options.changedFilesFile) {
    changedFiles = readChangedFiles(options.changedFilesFile);
  } else if (prNumber && REPOSITORY) {
    try {
      changedFiles = await fetchPullFiles(prNumber);
    } catch (error) {
      console.warn(`[smart-test] Could not fetch pull request files: ${error.message}`);
    }
  }

  const selection = selectTests({
    forceFull: options.forceFull,
    changedFiles,
    event,
    packages,
  });

  console.log(`[smart-test] mode=${selection.mode} reason=${selection.reason}`);

  if (selection.mode === 'scoped') {
    console.log(`[smart-test] packages=${selection.packages.join(', ')}`);
  }

  if (options.dryRun) {
    console.log(JSON.stringify({ selection, changedFiles }, null, 2));
    return;
  }

  fs.mkdirSync(REPORT_DIR, { recursive: true });
  fs.rmSync(JUNIT_PATH, { force: true });

  if (selection.mode === 'skip') {
    const text = `Skipping tests: ${selection.reason}\n`;
    fs.writeFileSync(LOG_PATH, text);
    process.stdout.write(text);
    writeStepSummary(text);
    return;
  }

  const startedAt = Date.now();
  let logText = '';
  const append = (text) => {
    logText += text;
    process.stdout.write(text);
  };

  const packageArgs = selection.mode === 'scoped'
    ? selection.packages.flatMap((name) => ['-p', name])
    : [];

  const nextestArgs = [
    'nextest',
    'run',
    '--profile',
    'ci',
    '--no-fail-fast',
    '--no-tests=pass',
    ...(selection.mode === 'full' ? ['--workspace'] : []),
    ...packageArgs,
  ];

  const nextest = await runStreaming('cargo', nextestArgs, append);
  const nextestLog = nextest.output;

  const doctestArgs = [
    'test',
    ...(selection.mode === 'full' ? ['--workspace'] : []),
    '--doc',
    '--no-fail-fast',
    ...packageArgs,
  ];

  const doctest = await runStreaming('cargo', doctestArgs, append);
  const doctestLog = doctest.output;
  const elapsedMs = Date.now() - startedAt;

  fs.writeFileSync(LOG_PATH, logText);

  const junitXml = fs.existsSync(JUNIT_PATH) ? fs.readFileSync(JUNIT_PATH, 'utf8') : '';
  const cases = parseJUnit(junitXml);
  const doctests = parseDoctests(doctestLog);
  const doctestOutputs = parseDoctestOutputs(doctestLog);
  const markdown = buildMarkdown({
    selection,
    cases,
    doctests,
    doctestOutputs,
    nextestCode: nextest.code,
    doctestCode: doctest.code,
    elapsedMs,
    nextestLog,
    doctestLog,
  });

  fs.writeFileSync(path.join(REPORT_DIR, 'summary.md'), markdown);
  writeStepSummary(markdown);
  console.log(`\n${markdown}`);

  if (prNumber) {
    try {
      await upsertComment(markdown, prNumber);
      console.log('[smart-test] Pull request comment updated');
    } catch (error) {
      console.warn(`[smart-test] Could not publish pull request comment: ${error.message}`);
    }
  }

  if (nextest.code !== 0 || doctest.code !== 0) {
    process.exitCode = 1;
  }
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (isMain) {
  main().catch((error) => {
    console.error(`[smart-test] ${error.stack || error.message}`);
    process.exitCode = 1;
  });
}

export {
  buildMarkdown,
  parseDoctestOutputs,
  parseDoctests,
  parseJUnit,
  selectTests,
};
