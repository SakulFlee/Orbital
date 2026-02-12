let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/Workspace/Orbital
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +67 ~/Workspace/Orbital/Crates/wit_runner/src/lib.rs
badd +8 ~/Workspace/Orbital/Crates/wit_runner/Cargo.toml
badd +3 ~/Workspace/Orbital/Crates/WITTest/wit/world.wit
argglobal
%argdel
$argadd .
edit ~/Workspace/Orbital/Crates/wit_runner/src/lib.rs
argglobal
balt ~/Workspace/Orbital/Crates/WITTest/wit/world.wit
setlocal foldmethod=expr
setlocal foldexpr=v:lua.require'astroui.folding'.foldexpr()
setlocal foldmarker={{{,}}}
setlocal foldignore=#
setlocal foldlevel=99
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldenable
1
sil! normal! zo
1
sil! normal! zo
15
sil! normal! zo
16
sil! normal! zo
24
sil! normal! zo
43
sil! normal! zo
73
sil! normal! zo
let s:l = 68 - ((30 * winheight(0) + 20) / 41)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 68
normal! 0
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
