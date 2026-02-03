#!/bin/bash
# Verify infrastructure is running correctly

set -e

echo "=== Verifying scroll-share infrastructure ==="

# Check namespace exists
echo -n "Checking namespace... "
if kubectl get namespace scroll-share &>/dev/null; then
    echo "OK"
else
    echo "FAIL: namespace 'scroll-share' not found"
    exit 1
fi

# Check CNPG operator is running
echo -n "Checking CloudNativePG operator... "
if kubectl get deployment cnpg-controller-manager -n cnpg-system &>/dev/null; then
    echo "OK"
else
    echo "FAIL: CloudNativePG operator not found"
    exit 1
fi

# Check PostgreSQL cluster is ready
echo -n "Checking PostgreSQL cluster... "
CLUSTER_STATUS=$(kubectl get cluster scroll-share-db -n scroll-share -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
if [ "$CLUSTER_STATUS" = "Cluster in healthy state" ]; then
    echo "OK"
else
    echo "PENDING (status: $CLUSTER_STATUS)"
fi

# Check nginx-ingress is running
echo -n "Checking nginx-ingress... "
if kubectl get pods -n ingress-nginx -l app.kubernetes.io/component=controller --field-selector=status.phase=Running 2>/dev/null | grep -q "Running"; then
    echo "OK"
else
    echo "PENDING"
fi

# Try to connect to PostgreSQL
echo -n "Checking PostgreSQL connectivity... "
POD=$(kubectl get pods -n scroll-share -l cnpg.io/cluster=scroll-share-db -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
if [ -n "$POD" ]; then
    # Use postgres superuser which has peer auth access via socket
    if kubectl exec -n scroll-share "$POD" -- bash -c 'psql -U postgres -d scrollshare -c "SELECT 1"' &>/dev/null; then
        echo "OK"
    else
        echo "FAIL: cannot connect"
    fi
else
    echo "PENDING: no pod found"
fi

# Check schemas exist
echo -n "Checking database schemas... "
if [ -n "$POD" ]; then
    SCHEMAS=$(kubectl exec -n scroll-share "$POD" -- bash -c "psql -U postgres -d scrollshare -t -c \"SELECT schema_name FROM information_schema.schemata WHERE schema_name IN ('auth', 'campaign', 'document', 'permission')\"" 2>/dev/null | grep -c -E "auth|campaign|document|permission" || echo "0")
    if [ "$SCHEMAS" -ge 4 ]; then
        echo "OK (all 4 schemas present)"
    else
        echo "PARTIAL ($SCHEMAS/4 schemas)"
    fi
else
    echo "SKIPPED: no pod"
fi

echo ""
echo "=== Infrastructure verification complete ==="
