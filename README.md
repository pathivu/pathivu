<img src="docs/img/pathivu.jpg" height="250" width="200" style="display: block;
  margin-left: auto;
  margin-right: auto;
  width: 50%;">    

# Pathivu: Logs you can search
Pathivu is a log aggreation system. It does all the indexing. All you have to shoot the query to find, why account tranfer is failed. It created for cost effective logging at scale.

It is primarly aimed for kubernetes but you can run on any deployment.

Not for production use yet. But, you can play around it.

[![asciicast](https://asciinema.org/a/ExDbYIsV2HODC9ORxe1UhVPtS.svg)](https://asciinema.org/a/ExDbYIsV2HODC9ORxe1UhVPtS)
## Usage
First run pathivu in your cluster.

```
kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/namespace.yaml

kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/deployment.yaml

kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/service.yaml
```
Then run pathivu fluentd plugin to ship logs to pathivu
```
kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/chola.yaml
```
# Use katchi to see logs
Katchi is cli tool to view logs
```
katchi logs --host=http://localhost:5180
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:06:58 +0530 IST, line: INFO: == Kubernetes addon reconcile completed at 2019-11-17T18:36:58+00:00 ==
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:02 +0530 IST, line: INFO: Leader election disabled.
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:03 +0530 IST, line: INFO: == Kubernetes addon ensure completed at 2019-11-17T18:37:03+00:00 ==
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:03 +0530 IST, line: INFO: == Reconciling with deprecated label ==
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:03 +0530 IST, line: error: no objects passed to apply

```
## Roadmap
- Multi node support.
- log shipper for several deploymets
- crash replayer.
- Feel free to GitHub issue for feature request.
