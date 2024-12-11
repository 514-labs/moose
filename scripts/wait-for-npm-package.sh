# scripts/wait-for-npm-package.sh
#!/bin/bash

PACKAGE_NAME=$1
VERSION=$2
MAX_ATTEMPTS=${3:-30}
SLEEP_SECONDS=${4:-10}

for ((i=1; i<=$MAX_ATTEMPTS; i++)); do
    echo "Attempt $i/$MAX_ATTEMPTS: Checking if $PACKAGE_NAME@$VERSION is available..."
    if npm view "$PACKAGE_NAME@$VERSION" version &> /dev/null; then
        echo "✅ Package $PACKAGE_NAME@$VERSION is available!"
        exit 0
    fi
    echo "Package not found. Waiting ${SLEEP_SECONDS} seconds..."
    sleep $SLEEP_SECONDS
done

echo "❌ Timeout waiting for package $PACKAGE_NAME@$VERSION"
exit 1
