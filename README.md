<img src="docs/img/pathivu.jpeg"  style="display: block;
  margin-left: auto;
  margin-right: auto;
  width: 50%;">    

# Pathivu: Logs you can search
<table align=left style='float: left; margin: 4px 10px 0px 0px; border: 1px solid #000000;'>

<tr>
  <td>License</td>
  <td>
    <a href="https://github.com/pathivu/pathivu/blob/master/LICENSE">
    <img alt="License" src="https://img.shields.io/badge/License-Apache%202.0-blue.svg">
    </a>
</td>
</tr>
<tr>
  <td>Build Status</td>
  <td>
    <a href="https://github.com/pathivu/pathivu/actions">
    <img alt="Build Status" src="https://github.com/pathivu/pathivu/workflows/Rust/badge.svg" />
    </a>
  </td>
</tr>
<tr>
	<td>Discord</td>
	<td>
		<a href="https://discord.gg/PGjRet">
		<img src="https://img.shields.io/discord/628383521450360842.svg?logo=discord" />
		</a>
	</td>
</tr>
</table>

--- 

Pathivu is powerful log ingestion and aggregation system. It's built from scratch by having cost-efficient and high write throughput in mind without trading log indexing. 

## Highlights
- Fast Ingestion
- Beautiful dashboard*
- Log tailing
- Log indexing
- Cost-Efficient
- Intuitive query language

**  Not yet released. It's on the roadmap



<p align="center"><img src="docs/tail.gif?raw=true"/></p>

## Usage

Follow the below steps to deploy Pathivu in kubernetes Cluster
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
