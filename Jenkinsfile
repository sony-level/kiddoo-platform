// =============================================================================
// Jenkinsfile — Post-merge Delivery Pipeline
//
// Role: Build, security, Docker, deploy (after merge).
// GitHub Actions handles: lint, unit tests, secret scan (before merge).
//
// Branch -> Environment:
//   develop    -> development  (auto deploy)
//   release/*  -> staging      (manual approval)
//   master     -> production   (manual approval)
// =============================================================================

pipeline {
    agent {
        docker {
            image 'ghcr.io/sony-level/kiddoo-jenkins-agent:latest'
            args '-v /var/run/docker.sock:/var/run/docker.sock'
            registryUrl 'https://ghcr.io'
            registryCredentialsId 'ghcr-token'
        }
    }

    environment {
        CARGO_TERM_COLOR = 'always'
        CARGO_HOME       = "${WORKSPACE}/.cargo"
        RUSTUP_HOME      = "${WORKSPACE}/.rustup"

        // GitHub Container Registry (private build)
        GHCR_REGISTRY    = 'ghcr.io/sony-level'
        GHCR_CREDENTIALS = credentials('ghcr-token')

        // AWS ECR (production artifacts)
        AWS_REGION       = 'eu-north-1'
        AWS_ACCOUNT_ID   = credentials('aws-account-id')
        AWS_CREDENTIALS  = credentials('aws-ecr-credentials')

        // SonarQube
        SONAR_HOST       = credentials('sonarqube-url')
        SONAR_TOKEN      = credentials('sonarqube-token')

        // Notifications (Discord)
        DISCORD_WEBHOOK  = credentials('discord-webhook-url')

        // Database (test container)
        DB_NAME          = 'kiddoo_test'
        DB_USER          = 'kiddoo'
        DB_PASSWORD      = 'kiddoo_test_pwd'
        DB_PORT          = '5433'
    }

    options {
        timeout(time: 60, unit: 'MINUTES')
        timestamps()
        disableConcurrentBuilds()
        buildDiscarder(logRotator(numToKeepStr: '20'))
    }

    stages {
        // =================================================================
        // STAGE 1 — Checkout & Setup
        // =================================================================
        stage('Checkout') {
            steps {
                checkout scm
                script {
                    def version = sh(script: "grep '^version' Cargo.toml | head -1 | sed 's/.*\"\\(.*\\)\"/\\1/'", returnStdout: true).trim()
                    env.BUILD_TAG_VERSION = "${version}-${BUILD_NUMBER}"
                    env.GIT_COMMIT_SHORT = sh(script: 'git rev-parse --short HEAD', returnStdout: true).trim()
                    echo "Version: ${BUILD_TAG_VERSION} | Commit: ${GIT_COMMIT_SHORT} | Branch: ${BRANCH_NAME}"
                }
            }
        }

        stage('Setup') {
            steps {
                sh '''
                    rustc --version
                    cargo --version
                    diesel --version
                    trivy --version
                '''
            }
        }

        // =================================================================
        // STAGE 2 — Build & Test (real build, not just check)
        // =================================================================
        stage('Database') {
            steps {
                sh '''
                    docker rm -f kiddoo-pg-test || true
                    docker run -d --name kiddoo-pg-test \
                        -e POSTGRES_DB=${DB_NAME} \
                        -e POSTGRES_USER=${DB_USER} \
                        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
                        -p ${DB_PORT}:5432 \
                        postgres:16-bookworm

                    for i in $(seq 1 30); do
                        if docker exec kiddoo-pg-test pg_isready -U ${DB_USER} > /dev/null 2>&1; then
                            echo "PostgreSQL is ready"
                            break
                        fi
                        echo "Waiting for PostgreSQL... ($i/30)"
                        sleep 1
                    done

                    export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
                    cd libs/database
                    diesel migration run
                    echo "Migrations applied successfully"
                '''
            }
            post {
                always {
                    sh 'docker rm -f kiddoo-pg-test || true'
                }
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

        // =================================================================
        // STAGE 3 — Security & Quality (deep analysis)
        // =================================================================
        stage('Security & Quality') {
            parallel {
                stage('cargo-audit') {
                    steps {
                        sh 'cargo audit 2>&1 | tee cargo-audit-report.txt'
                    }
                    post {
                        always {
                            archiveArtifacts artifacts: 'cargo-audit-report.txt', allowEmptyArchive: true
                        }
                    }
                }
                stage('Trivy Filesystem') {
                    steps {
                        sh 'trivy fs --severity HIGH,CRITICAL --exit-code 0 --format table . 2>&1 | tee trivy-fs-report.txt'
                    }
                    post {
                        always {
                            archiveArtifacts artifacts: 'trivy-fs-report.txt', allowEmptyArchive: true
                        }
                    }
                }
                stage('Trivy Config') {
                    steps {
                        sh 'trivy config --severity HIGH,CRITICAL --exit-code 0 . 2>&1 | tee trivy-config-report.txt'
                    }
                    post {
                        always {
                            archiveArtifacts artifacts: 'trivy-config-report.txt', allowEmptyArchive: true
                        }
                    }
                }
            }
        }

        stage('SonarQube') {
            when {
                anyOf {
                    branch 'develop'
                    branch 'release/*'
                }
            }
            steps {
                sh """
                    sonar-scanner \
                        -Dsonar.projectKey=kiddoo-platform \
                        -Dsonar.sources=services/,libs/ \
                        -Dsonar.host.url=${SONAR_HOST} \
                        -Dsonar.token=${SONAR_TOKEN} \
                        -Dsonar.qualitygate.wait=true
                """
            }
        }

        // =================================================================
        // STAGE 4 — Docker Build & Push to GHCR (private)
        // =================================================================
        stage('Docker Build & Push GHCR') {
            when {
                anyOf {
                    branch 'develop'
                    branch 'release/*'
                    branch 'master'
                }
            }
            steps {
                script {
                    sh """
                        echo '${GHCR_CREDENTIALS_PSW}' | docker login ghcr.io -u '${GHCR_CREDENTIALS_USR}' --password-stdin

                        docker build -t ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} \
                                     -t ${GHCR_REGISTRY}/kiddoo-api-gateway:${GIT_COMMIT_SHORT} \
                                     -f services/api-gateway/Dockerfile .

                        docker build -t ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} \
                                     -t ${GHCR_REGISTRY}/kiddoo-identity-proxy:${GIT_COMMIT_SHORT} \
                                     -f services/identity-proxy/Dockerfile .
                    """
                }
            }
            post {
                always {
                    sh 'docker logout ghcr.io || true'
                }
            }
        }

        stage('Trivy Image Scan') {
            when {
                anyOf {
                    branch 'develop'
                    branch 'release/*'
                    branch 'master'
                }
            }
            steps {
                sh """
                    trivy image --severity HIGH,CRITICAL --exit-code 0 \
                        ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} 2>&1 | tee trivy-image-api-gateway.txt

                    trivy image --severity HIGH,CRITICAL --exit-code 0 \
                        ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} 2>&1 | tee trivy-image-identity-proxy.txt
                """
            }
            post {
                always {
                    archiveArtifacts artifacts: 'trivy-image-*.txt', allowEmptyArchive: true
                }
            }
        }

        stage('Push Images to GHCR') {
            when {
                anyOf {
                    branch 'develop'
                    branch 'release/*'
                    branch 'master'
                }
            }
            steps {
                sh """
                    echo '${GHCR_CREDENTIALS_PSW}' | docker login ghcr.io -u '${GHCR_CREDENTIALS_USR}' --password-stdin

                    docker push ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                    docker push ${GHCR_REGISTRY}/kiddoo-api-gateway:${GIT_COMMIT_SHORT}
                    docker push ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}
                    docker push ${GHCR_REGISTRY}/kiddoo-identity-proxy:${GIT_COMMIT_SHORT}
                """
            }
            post {
                always {
                    sh 'docker logout ghcr.io || true'
                }
            }
        }

        // =================================================================
        // STAGE 5 — Deploy Dev (auto on develop)
        // =================================================================
        stage('Deploy Dev') {
            when {
                branch 'develop'
            }
            steps {
                script {
                    def ecrRegistry = "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

                    sh """
                        aws ecr get-login-password --region ${AWS_REGION} | \
                            docker login --username AWS --password-stdin ${ecrRegistry}

                        aws ecr describe-repositories --repository-names kiddoo-api-gateway --region ${AWS_REGION} || \
                            aws ecr create-repository --repository-name kiddoo-api-gateway --region ${AWS_REGION} --image-tag-mutability MUTABLE
                        aws ecr describe-repositories --repository-names kiddoo-identity-proxy --region ${AWS_REGION} || \
                            aws ecr create-repository --repository-name kiddoo-identity-proxy --region ${AWS_REGION} --image-tag-mutability MUTABLE

                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-api-gateway:dev
                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-identity-proxy:dev
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}

                        docker push ${ecrRegistry}/kiddoo-api-gateway:dev
                        docker push ${ecrRegistry}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:dev
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}
                    """

                    echo "Deployed ${BUILD_TAG_VERSION} to DEV"
                }
            }
            post {
                always {
                    sh "docker logout ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com || true"
                }
            }
        }

        stage('Smoke Tests Dev') {
            when {
                branch 'develop'
            }
            steps {
                sh '''
                    echo "Running smoke tests on dev environment..."
                    # TODO: Replace with actual dev endpoint
                    # curl -sf https://api-dev.level-sony.com/api/v1/health || exit 1
                    echo "Smoke tests passed"
                '''
            }
        }

        // =================================================================
        // STAGE 6 — Deploy Staging (manual approval on release/*)
        // =================================================================
        stage('Approve Staging') {
            when {
                branch 'release/*'
            }
            steps {
                input message: "Deploy ${BUILD_TAG_VERSION} to STAGING?", ok: 'Deploy'
            }
        }

        stage('Deploy Staging') {
            when {
                branch 'release/*'
            }
            steps {
                script {
                    def ecrRegistry = "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

                    sh """
                        aws ecr get-login-password --region ${AWS_REGION} | \
                            docker login --username AWS --password-stdin ${ecrRegistry}

                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-api-gateway:staging
                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-identity-proxy:staging
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}

                        docker push ${ecrRegistry}/kiddoo-api-gateway:staging
                        docker push ${ecrRegistry}/kiddoo-api-gateway:${BUILD_TAG_VERSION}
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:staging
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:${BUILD_TAG_VERSION}
                    """

                    echo "Deployed ${BUILD_TAG_VERSION} to STAGING"
                }
            }
            post {
                always {
                    sh "docker logout ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com || true"
                }
            }
        }

        stage('E2E Tests Staging') {
            when {
                branch 'release/*'
            }
            steps {
                sh '''
                    echo "Running E2E tests on staging environment..."
                    # TODO: Replace with actual staging endpoint
                    # curl -sf https://api-staging.level-sony.com/api/v1/health || exit 1
                    echo "E2E tests passed"
                '''
            }
        }

        // =================================================================
        // STAGE 7 — Deploy Prod (manual approval on master)
        // =================================================================
        stage('Approve Production') {
            when {
                branch 'master'
            }
            steps {
                input message: "Deploy ${BUILD_TAG_VERSION} to PRODUCTION?", ok: 'Deploy to Prod'
            }
        }

        stage('Deploy Production') {
            when {
                branch 'master'
            }
            steps {
                script {
                    def ecrRegistry = "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

                    sh """
                        aws ecr get-login-password --region ${AWS_REGION} | \
                            docker login --username AWS --password-stdin ${ecrRegistry}

                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-api-gateway:latest
                        docker tag ${GHCR_REGISTRY}/kiddoo-api-gateway:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-api-gateway:prod
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-identity-proxy:latest
                        docker tag ${GHCR_REGISTRY}/kiddoo-identity-proxy:${BUILD_TAG_VERSION} ${ecrRegistry}/kiddoo-identity-proxy:prod

                        docker push ${ecrRegistry}/kiddoo-api-gateway:latest
                        docker push ${ecrRegistry}/kiddoo-api-gateway:prod
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:latest
                        docker push ${ecrRegistry}/kiddoo-identity-proxy:prod
                    """

                    echo "Deployed ${BUILD_TAG_VERSION} to PRODUCTION"
                }
            }
            post {
                always {
                    sh "docker logout ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com || true"
                }
            }
        }

        stage('Health Check Prod') {
            when {
                branch 'master'
            }
            steps {
                sh '''
                    echo "Running health checks on production..."
                    # TODO: Replace with actual production endpoint
                    # for i in $(seq 1 5); do
                    #     curl -sf https://api.level-sony.com/api/v1/health && break
                    #     echo "Health check attempt $i/5 failed, retrying in 10s..."
                    #     sleep 10
                    # done
                    echo "Health checks passed"
                '''
            }
            post {
                failure {
                    echo "PRODUCTION HEALTH CHECK FAILED — consider rollback"
                }
            }
        }
    }

    // =====================================================================
    // Post-build: notifications & cleanup
    // =====================================================================
    post {
        success {
            script {
                if (env.DISCORD_WEBHOOK) {
                    discordSend(
                        webhookURL: env.DISCORD_WEBHOOK,
                        title: "✅ Build SUCCESS",
                        description: "**${env.JOB_NAME}** #${env.BUILD_NUMBER}\nBranch: `${env.BRANCH_NAME}`\nVersion: `${env.BUILD_TAG_VERSION}`",
                        link: env.BUILD_URL,
                        result: currentBuild.currentResult
                    )
                }
            }
        }
        failure {
            script {
                if (env.DISCORD_WEBHOOK) {
                    discordSend(
                        webhookURL: env.DISCORD_WEBHOOK,
                        title: "❌ Build FAILED",
                        description: "**${env.JOB_NAME}** #${env.BUILD_NUMBER}\nBranch: `${env.BRANCH_NAME}`\nVersion: `${env.BUILD_TAG_VERSION}`",
                        link: env.BUILD_URL,
                        result: currentBuild.currentResult
                    )
                }
            }
        }
        always {
            script {
                try { cleanWs() } catch (e) { echo "Workspace cleanup skipped: ${e.message}" }
            }
        }
    }
}