pipeline {
    agent {
        dockerfile {
            filename 'jenkins/Dockerfile.agent'
            args '-v /var/run/docker.sock:/var/run/docker.sock'
        }
    }

    environment {
        CARGO_TERM_COLOR = 'always'
        CARGO_HOME       = "${WORKSPACE}/.cargo"
        RUSTUP_HOME      = "${WORKSPACE}/.rustup"

        // GitHub Container Registry (private)
        GHCR_REGISTRY    = 'ghcr.io/sony-level'
        GHCR_CREDENTIALS = credentials('ghcr-token')

        // AWS ECR (production artifacts)
        AWS_REGION       = 'eu-north-1'
        AWS_ACCOUNT_ID   = credentials('aws-account-id')
        AWS_CREDENTIALS  = credentials('aws-ecr-credentials')

        SLACK_WEBHOOK    = credentials('slack-webhook-url')

        // Database (test container)
        DB_NAME          = 'kiddoo_test'
        DB_USER          = 'kiddoo'
        DB_PASSWORD      = 'kiddoo_test_pwd'
        DB_PORT          = '5433'
    }

    options {
        timeout(time: 45, unit: 'MINUTES')
        timestamps()
        disableConcurrentBuilds()
        buildDiscarder(logRotator(numToKeepStr: '10'))
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
                sh 'git submodule update --init --recursive'
            }
        }

        stage('Setup') {
            steps {
                sh '''
                    rustc --version
                    cargo --version
                    rustup component list --installed
                '''
            }
        }

        stage('Lint') {
            parallel {
                stage('Format Check') {
                    steps {
                        sh 'cargo fmt --all -- --check'
                    }
                }
                stage('Clippy') {
                    steps {
                        sh 'cargo clippy --workspace -- -D warnings'
                    }
                }
            }
        }

        stage('Database') {
            steps {
                sh '''
                    # Start PostgreSQL container for tests
                    docker rm -f kiddoo-pg-test || true
                    docker run -d --name kiddoo-pg-test \
                        -e POSTGRES_DB=${DB_NAME} \
                        -e POSTGRES_USER=${DB_USER} \
                        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
                        -p ${DB_PORT}:5432 \
                        postgres:16-bookworm

                    # Wait for PostgreSQL to be ready
                    for i in $(seq 1 30); do
                        if docker exec kiddoo-pg-test pg_isready -U ${DB_USER} > /dev/null 2>&1; then
                            echo "PostgreSQL is ready"
                            break
                        fi
                        echo "Waiting for PostgreSQL... ($i/30)"
                        sleep 1
                    done

                    # Run Diesel migrations
                    export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
                    cd libs/database
                    diesel migration run
                    echo "Migrations applied successfully"
                '''
            }
        }

        stage('Test') {
            environment {
                DATABASE_URL = "postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
            }
            steps {
                sh 'cargo test --workspace --no-fail-fast 2>&1 | tee test-output.txt'
            }
            post {
                always {
                    archiveArtifacts artifacts: 'test-output.txt', allowEmptyArchive: true
                }
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release --workspace'
            }
            post {
                success {
                    archiveArtifacts artifacts: 'target/release/api-gateway,target/release/identity-proxy,target/release/orchestrator', allowEmptyArchive: true
                }
            }
        }

        stage('Security') {
            steps {
                sh 'cargo audit || true'
            }
        }

        // ---- Build & push images privately to ghcr.io ----
        stage('Package to GHCR') {
            when {
                anyOf {
                    branch 'main'
                    branch 'develop'
                    branch 'release/*'
                    branch 'feature/*'
                }
            }
            steps {
                script {
                    def version = sh(script: "grep '^version' Cargo.toml | head -1 | sed 's/.*\"\\(.*\\)\"/\\1/'", returnStdout: true).trim()
                    env.BUILD_TAG_VERSION = "${version}-${BUILD_NUMBER}"

                    sh """
                        echo '${GHCR_CREDENTIALS_PSW}' | docker login ghcr.io -u '${GHCR_CREDENTIALS_USR}' --password-stdin

                        docker build -t ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} \
                                     -t ${GHCR_REGISTRY}/kiddoo-api-gateway:latest \
                                     -f services/api-gateway/Dockerfile .

                        docker build -t ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} \
                                     -t ${GHCR_REGISTRY}/kiddoo-identity-proxy:latest \
                                     -f services/identity-proxy/Dockerfile .

                        docker push ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker push ${GHCR_REGISTRY}/kiddoo-api-gateway:latest
                        docker push ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}
                        docker push ${GHCR_REGISTRY}/kiddoo-identity-proxy:latest
                    """
                }
            }
            post {
                always {
                    sh 'docker logout ghcr.io || true'
                }
            }
        }

        // ---- Promote to AWS ECR only if all tests passed ----
        stage('Promote to AWS ECR') {
            when {
                anyOf {
                    branch 'main'
                    branch 'release/*'
                }
            }
            steps {
                script {
                    def ecrRegistry = "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

                    sh """
                        aws ecr get-login-password --region ${AWS_REGION} | \
                            docker login --username AWS --password-stdin ${ecrRegistry}

                        # Create repositories if they don't exist
                        aws ecr describe-repositories --repository-names kiddoo-api-gateway --region ${AWS_REGION} || \
                            aws ecr create-repository --repository-name kiddoo-api-gateway --region ${AWS_REGION} --image-tag-mutability MUTABLE
                        aws ecr describe-repositories --repository-names kiddoo-identity-proxy --region ${AWS_REGION} || \
                            aws ecr create-repository --repository-name kiddoo-identity-proxy --region ${AWS_REGION} --image-tag-mutability MUTABLE

                        # Tag from GHCR and push to ECR
                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} \
                                   ${ecrRegistry}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:latest \
                                   ${ecrRegistry}/kiddoo-api-gateway:latest

                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} \
                                   ${ecrRegistry}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:latest \
                                   ${ecrRegistry}/kiddoo-identity-proxy:latest

                        docker push ${ecrRegistry}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker push ${ecrRegistry}/kiddoo-api-gateway:latest
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:latest
                    """

                    echo "Promoted ${BUILD_TAG_VERSION} to AWS ECR: ${ecrRegistry}"
                }
            }
            post {
                always {
                    sh "docker logout ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com || true"
                }
            }
        }

        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                script {
                    def ecrRegistry = "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
                    echo "Deploying ${BUILD_TAG_VERSION} from ${ecrRegistry} to production"
                }
            }
        }
    }

    post {
        success {
            script {
                if (env.SLACK_WEBHOOK) {
                    slackSend(
                        color: 'good',
                        message: "Build SUCCESS: ${env.JOB_NAME} #${env.BUILD_NUMBER} (<${env.BUILD_URL}|Open>)"
                    )
                }
            }
        }
        failure {
            script {
                if (env.SLACK_WEBHOOK) {
                    slackSend(
                        color: 'danger',
                        message: "Build FAILED: ${env.JOB_NAME} #${env.BUILD_NUMBER} (<${env.BUILD_URL}|Open>)"
                    )
                }
            }
        }
        always {
            sh 'docker rm -f kiddoo-pg-test || true'
            cleanWs()
        }
    }
}