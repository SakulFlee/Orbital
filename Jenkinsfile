podTemplate(label: "k8s",
    containers: [
        containerTemplate(name: 'rust', image: 'rust:latest', ttyEnabled: true, command: 'cat'),
    ]) {
    node("k8s") {
        stage('Rust') {
            parallel {
                stage('Check') {
                    sh 'cargo check'
                }
                stage('Test') {
                    sh 'cargo test'
                }
            }
        }
    }
}
