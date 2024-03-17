# Redpanda Install on GKE

Derived from: 
- Redpanda on Google Cloud (w/ Helm & Kubernetes) https://www.youtube.com/watch?v=DbmbXZVL7lw
- Redpanda docs: https://docs.redpanda.com/current/deploy/deployment-option/self-hosted/kubernetes/gke-guide

## Add Redpanda repository

```sh
$ helm repo add redpanda https://charts.redpanda.com
```

## Add Jetstack repo

```sh
$ helm repo add jetstack https://charts.jetstack.io
```

## Update repos

```sh
$ helm repo update
```

## Install cert-manager
```sh
$ helm install cert-manager jetstack/cert-manager --set installCRDs=true --namespace cert-manager --create-namespace
```

## Install the Redpanda Operator custom resource definitions (CRDs):
```sh
$ kubectl kustomize "https://github.com/redpanda-data/redpanda-operator//src/go/k8s/config/crd?ref=v2.1.14-23.3.4" > kustomize.yaml
$ kubectl apply -f kustomize.yaml
```

## Deploy the Redpanda Operator:
```sh
$ helm upgrade --install redpanda-controller redpanda/operator --namespace redpanda --set image.tag=v2.1.14-23.3.4 --create-namespace --timeout 1h
```

## Install a Redpanda custom resource

### redpanda-cluster.yaml
```yaml
apiVersion: cluster.redpanda.com/v1alpha1
kind: Redpanda
metadata:
  name: redpanda
spec:
  chartRef: {}
  clusterSpec:
    external:
      domain: {domain.com}
    auth:
      sasl:
        enabled: true
        users:
          - name: {fake_name}
            password: {fake_password}
    storage:
      persistentVolume:
        enabled: true
        #storageClass: csi-driver-lvm-striped-xfs
```

```sh
$ kubectl apply -f redpanda-cluster.yaml --namespace redpanda
```

## Verify deployment
```sh
$ kubectl get pod --namespace redpanda -o=custom-columns=NODE:.spec.nodeName,POD_NAME:.metadata.name -l app.kubernetes.io/component=redpanda-statefulset
```

## View bound persistent storage
```sh
$ kubectl get persistentvolumeclaim --namespace redpanda -o custom-columns=NAME:.metadata.name,STATUS:.status.phase,STORAGECLASS:.spec.storageClassName
```

---

## Install rpk on mac:

```
$ brew install redpanda-data/tap/redpanda
```

## Create a user
https://docs.redpanda.com/current/deploy/deployment-option/self-hosted/kubernetes/gke-guide/?tab=tabs-3-macos#create-a-user

```sh
$ kubectl --namespace redpanda exec -ti redpanda-0 -c redpanda -- \
rpk acl user create moose-account \
-p {fake_password}
```

