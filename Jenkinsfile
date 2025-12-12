podTemplate(label: "k8s",
    containers: [
        containerTemplate(name: 'rust', image: 'rust:latest', ttyEnabled: true, command: 'cat'),
    ]) {
    node("k8s") {
        stage('Check') {
            container('rust') {
                sh 'cargo check'
            }
        }
        stage('Test') {
            container('rust') {
                 sh 'cargo test'
            }
        }
    }
}
