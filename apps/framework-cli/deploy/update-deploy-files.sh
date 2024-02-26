mkdir -p deployment
cp -R app ./deployment
#cp node_modules/@514labs/moose-cli-darwin-arm64/bin/moose-cli ./deployment
#cp /Users/cjus/dev/moose/apps/framework-cli/target/debug/moose-cli ./deployment
cp /Users/cjus/dev/moose/apps/framework-cli/target/release/moose-cli ./deployment
cp /Users/cjus/dev/moose/apps/framework-cli/deploy/pseudobuild.sh .
cp /Users/cjus/dev/moose/apps/framework-cli/deploy/buildx_setup.sh .
cp /Users/cjus/dev/moose/apps/framework-cli/deploy/Dockerfile.deployment ./deployment/Dockerfile
