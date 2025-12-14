pipeline {
    agent {
        kubernetes {
            label 'k8s'
            defaultContainer 'rust'
            yaml """
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: rust
    image: rust:latest
    command: ["cat"]
    tty: true
"""
        }
    }

    stages {
        stage('Parallel Tasks') {
            parallel {
                stage('Check') {
                    steps {
                        container('rust') {
                            sh 'cargo check'
                        }
                    }
                }
                stage('Test') {
                    steps {
                        container('rust') {
                            sh 'cargo test'
                        }
                    }
                }
            }
        }
    }
}
