if [ -z "$1" ]
then
      echo "You must specify the dockerhub repository as an argument. Example: ./pseudobuild.sh yourdockerhubusername"
      exit 1
fi
cd deployment

VERSION="0.3.93"
output=$(uname -a)
os=$(echo $output | cut -d ' ' -f1)

if [[ $os == "Linux" ]]; then
    if [[ $output == *"x86_64"* ]]; then
        docker buildx build --build-arg DOWNLOAD_URL=https://github.com/514-labs/moose/releases/download/v${VERSION}/moose-cli-x86_64-unknown-linux-gnu --platform=linux/amd64 --push --no-cache -t $1/moose-deployment:latest .
    elif [[ $output == *"arm64"* ]]; then
        docker buildx build --build-arg DOWNLOAD_URL=https://github.com/514-labs/moose/releases/download/v${VERSION}/moose-cli-aarch64-unknown-linux-gnu --platform=linux/arm64 --push --no-cache -t $1/moose-deployment:latest .
    fi
elif [[ $os == "Darwin" ]]; then
    # Docker doesn't support creating macOS (darwin) containers as it uses Linux kernel features. 
    # Therefore, there is no `--platform` value for creating darwin x86_64 and aarch64 Docker images. 
    if [[ $output == *"x86_64"* ]]; then
        docker buildx build --build-arg DOWNLOAD_URL=https://github.com/514-labs/moose/releases/download/v${VERSION}/moose-cli-x86_64-unknown-linux-gnu --platform=linux/amd64 --push --no-cache -t $1/moose-deployment:latest .
    elif [[ $output == *"arm64"* ]]; then
        docker buildx build --build-arg DOWNLOAD_URL=https://github.com/514-labs/moose/releases/download/v${VERSION}/moose-cli-aarch64-unknown-linux-gnu --platform=linux/arm64 --push --no-cache -t $1/moose-deployment:latest .
    fi
fi
