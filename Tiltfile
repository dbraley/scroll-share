# scroll-share Tiltfile
# Orchestrates local K8s development with live reload

# ============================================================================
# Configuration
# ============================================================================

# Allow deploying to kind cluster
allow_k8s_contexts('kind-scroll-share')

# ============================================================================
# Namespace
# ============================================================================

k8s_yaml('deploy/k8s/namespace.yaml')

# ============================================================================
# Infrastructure: CloudNativePG Operator
# ============================================================================

# Install CloudNativePG operator if not present
# This is idempotent - it won't reinstall if already present
local_resource(
    'cnpg-operator',
    cmd='kubectl get deployment -n cnpg-system cnpg-controller-manager || kubectl apply --server-side -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.22/releases/cnpg-1.22.1.yaml',
    deps=[],
    labels=['infrastructure'],
)

# Wait for CNPG operator to be ready before deploying the cluster
local_resource(
    'cnpg-operator-ready',
    cmd='kubectl wait --for=condition=Available deployment/cnpg-controller-manager -n cnpg-system --timeout=120s',
    resource_deps=['cnpg-operator'],
    labels=['infrastructure'],
)

# ============================================================================
# Infrastructure: PostgreSQL (via CloudNativePG)
# ============================================================================

# Deploy the CloudNativePG Cluster CR after operator is ready
# Using local_resource because Tilt doesn't natively track CRDs
local_resource(
    'postgres-cluster',
    cmd='kubectl apply -f deploy/k8s/postgres/cluster.yaml',
    deps=['deploy/k8s/postgres/cluster.yaml'],
    resource_deps=['cnpg-operator-ready'],
    labels=['infrastructure'],
)

# Wait for PostgreSQL cluster to be ready
local_resource(
    'postgres-ready',
    cmd='kubectl wait --for=condition=Ready cluster/scroll-share-db -n scroll-share --timeout=180s',
    resource_deps=['postgres-cluster'],
    labels=['infrastructure'],
)

# ============================================================================
# Infrastructure: nginx-ingress
# ============================================================================

local_resource(
    'nginx-ingress',
    cmd='kubectl get namespace ingress-nginx || kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml',
    deps=[],
    labels=['infrastructure'],
)

local_resource(
    'nginx-ingress-ready',
    cmd='kubectl wait --namespace ingress-nginx --for=condition=ready pod --selector=app.kubernetes.io/component=controller --timeout=120s',
    resource_deps=['nginx-ingress'],
    labels=['infrastructure'],
)

# ============================================================================
# Services (to be added in later features)
# ============================================================================

# auth-service will be added in Feature 2
# campaign-service will be added in Phase 2
# document-service will be added in Phase 3
# permission-service will be added in Phase 4
# web will be added in Feature 4/5
