# Preparing your moose project for cloud deployment

### Firstly, create a new moose project:

```bash
npx create-moose-app my-moose-app3
cd my-moose-app3
```

### Build local docker containers:
* note make sure your have docker hub installed locally.

```bash
npx @514labs/moose-cli docker init
npx @514labs/moose-cli docker build
```

### Push local docker containers to your remote container repository:
* Tip: see sample push.sh script near the end of this document.

```bash
./push.sh cjus514
```

---

### Sample run:

Run the moose-cli docker init and docker build commands.

```bash
> npx @514labs/moose-cli docker init
Need to install the following packages:
@514labs/moose-cli@0.3.136
Ok to proceed? (y) y
          Init Loading config...
  Successfully created dockerfile

> npx @514labs/moose-cli docker build
          Init Loading config...
⢹ Creating docker linux/amd64 image
⡧ Creating docker linux/arm64 image
  Successfully created docker image for deployment
```

Next check that you need have two new docker images.

```bash
> docker images
REPOSITORY                                              TAG               IMAGE ID       CREATED              SIZE
moose-df-deployment-aarch64-unknown-linux-gnu           0.3.136           c50674c7a68a   About a minute ago   155MB
moose-df-deployment-x86_64-unknown-linux-gnu            0.3.136           e5b449d3dea3   About a minute ago   163MB
```

Use the handly push.sh script (see sample at end of this doc)
Note: change your docker hub username from `cjus514` to your account name.

```bash
> ./push.sh cjus514
Using version: 0.3.136
The push refers to repository [docker.io/cjus514/moose-df-deployment-aarch64-unknown-linux-gnu]
07558ca75451: Pushed
adc2f9f5b459: Pushed
64548c967302: Pushed
5f70bf18a086: Mounted from 514labs/moose-df-deployment-x86_64-unknown-linux-gnu
9ccfa9ea4367: Pushed
5ab2305446ba: Pushed
2f4308ac27fb: Pushed
515a95308a05: Pushed
07bbd89cd181: Pushed
0.3.136: digest: sha256:b653c2b99981004f42289c2467612b177706dbe27f10ebc55f86a32900a35ff3 size: 2408
The push refers to repository [docker.io/cjus514/moose-df-deployment-x86_64-unknown-linux-gnu]
b15e86f0517f: Pushed
c9ad293fae05: Pushed
57a3fee6d080: Pushed
5f70bf18a086: Mounted from cjus514/moose-df-deployment-aarch64-unknown-linux-gnu
e6e6d825b6e9: Pushed
9431edc9c228: Pushed
6ca745d58965: Pushed
0b0f84d039f4: Pushed
05a22b5033fd: Pushed
0.3.136: digest: sha256:ec8969a14dcd312bd9e2590111fb60836ea95bbb2906ca00215527668c97cfa1 size: 2408
```

---

## Sample push.sh script

```bash
#!/bin/bash

version=$2

if [ -z "$1" ]
then
      echo "You must specify the dockerhub repository as an argument. Example: ./push.sh container-repo-name"
      echo "Note: you can also provide a second argument to supply a specific version tag - otherwise this script will use the same version as the latest moose-cli on Github."
      exit 1
fi

if [ -z "$2" ]
then
      output=$(npx @514labs/moose-cli -V)
      version=$(echo "$output" | sed -n '2p' | awk '{print $2}')
fi

echo "Using version: $version"
arch="moose-df-deployment-aarch64-unknown-linux-gnu"
docker tag $arch:$version $1/$arch:$version
docker push $1/$arch:$version

arch="moose-df-deployment-x86_64-unknown-linux-gnu"
docker tag $arch:$version $1/$arch:$version
docker push $1/$arch:$version
```