cd deployment
docker buildx create --use
docker buildx build --platform=linux/amd64,linux/arm64 --push --no-cache -t $1/moose-deployment:latest .
