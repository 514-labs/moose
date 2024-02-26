if [ -z "$1" ]
then
      echo "You must specify the dockerhub repository as an argument. Example: ./pseudobuild.sh yourdockerhubusername"
      exit 1
fi
cd deployment
docker buildx build --platform=linux/amd64,linux/arm64 --push --no-cache -t $1/moose-deployment:latest .
