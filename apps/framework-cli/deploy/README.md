## Build

```
docker build -t 514labs/moose-fullstack:0.0.0 -f ./Dockerfile.fullstack --build-arg FRAMEWORK_VERSION=0.3.61 .
```

## Run (no data persisted)

```
docker run --memory=4g 514labs/moose-fullstack:0.0.0
```
