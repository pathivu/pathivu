<div align="center">
	<img align="center" width="30%" src="https://github.com/pathivu/pathivu/blob/master/docs/img/pathivu.jpeg?raw=true">
	<h1>
		Pathivu: Logs you can search
	</h1>
</div>


 Pathivu is a powerful and lightweight log ingestion and aggregation system.  It offers cost-efficiency and high throughput without trading away log indexing. It is perfect for cloud native workloads.



<p align="center"><img width="90%" src="https://github.com/pathivu/pathivu/blob/master/docs/tail.gif?raw=true"/></p>

<div align="center">
<a href="https://github.com/pathivu/pathivu/blob/master/LICENSE">
	 <img alt="License" src="https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square&logo=appveyor">
</a> <a href="https://discord.gg/PGjRet">
	  <img src="https://img.shields.io/discord/628383521450360842.svg?logo=discord" />
</a> <a href="https://docs.pathivu.io/#/">
	   <img alt="View Documentation" src="https://img.shields.io/badge/docs-view%20documentation-orange?style=flat-square&logo=appveyor" />
</a>
<a href="https://github.com/pathivu/pathivu/actions">
	   <img alt="Build Status" src="https://github.com/pathivu/pathivu/workflows/Rust/badge.svg" />
</a>
<a href="https://pathivu.io/#/">
	   <img alt="View Website" src="https://img.shields.io/badge/website-view%20website-yellowgreen?style=flat-square&logo=appveyor" />
</a>
</div>

## Index
- [Highlights](#highlights)
- [Architecture](#architecture)
- [Pathivu Server](#pathivu-server)
- [Pathivu Web](#pathivu-web)
- [Katchi](#katchi)
- [Usage](#usage)
- [Documentation](https://docs.pathivu.io/#/)
- [Website](https://pathivu.io)

<br>
<br>


## Highlights
- [X] Fast Ingestion
- [ ] Beautiful dashboard*
- [X] Log tailing
- [X] Log indexing
- [X] Cost-Efficient
- [X] Intuitive query language
- [X] Multi-threaded log ingestion
- [X] Structured logging replayer
- [X] Log retention 

<br>
<br>

## Architecture

A *fluentd* service running on a Kubernetes node can be used to ship logs to the Pathivu server. The server then exposes two types of interfaces, namely web and CLI. The following are the components of Pathivu:

* **Pathivu server**: A high performant log ingestion server where you can ingest logs at terabyte scale and query logs in efficient way, with both HTTP(s) as well as gRPC endpoints available. Read more about it [here](#pathivu-server)

* **Katchi CLI**:  Katchi is the command line interface for Pathivu. Read more about it [here](#katchi)

* **Pathivu Web**: It is a web based user interface for interacting with Pathivu. It makes the querying experience simplistic and intuitive. Read more about it [here](#pathivu-web.)


<p align="center">
<img src="https://user-images.githubusercontent.com/30529572/74427447-1c3f2400-4e7d-11ea-950e-292723957bbb.png" alt="Pathivu Architecture" width="75%"/>
</p>

<br>
<br>

## Pathivu Server
Pathivu server is a high performant log ingestion server where you can ingest logs at terabyte scale and query logs in efficient way, with both HTTP(s) as well as gRPC endpoints available. It offers a gRPC service for fast log ingestion and an HTTP(s) backend for log querying and aggregation. By default, log ingestion runs on gRPC port `6180` and querying on HTTP(s) port `5180`. 

Pathivu server has a *fluentd* connector that can be used to ship logs to pathivu from a Kubernetes node. For more information on architecture, see [this](#architecture).

<br>
<br>


## Pathivu Web
Pathivu Web is a web based user interface for interacting with Pathivu. It makes the querying experience simplistic and intuitive. You can use Pathivu's [simplistic query language](https://docs.pathivu.io/#/query) to interact with the server right from your browser.

<p align="center"> 
<img alt="Pathivu Web" src="https://user-images.githubusercontent.com/30529572/74428923-f8311200-4e7f-11ea-8373-9fecc10a6404.png" width="80%"/>
</p>

<br>
<br>

## Katchi
Katchi CLI can be used to interact with the pathivu server instance. It has the following functionalities:

- [X] Log service discovery
- [X] Querying 
- [X] Tailing
- [X] Log output

For example, the distinct command also provides a feature to count the number of distinct logs matched. It is a very powerful query which can handle data at a terabyte scale. For the following JSON, the `distinct(level)`command will give you a list of all distinct levels in the logs. 

```json
{
  "data": [
    {
      "ts": 3,
      "entry": {
        "details": {
          "error_code": "500",
          "message": "Invalid URI"
        },
        "level": "warn",
        "from": "backend"
      },
      "source": "demo"
    },
    {
      "ts": 2,
      "entry": {
        "details": {
          "error_code": "500",
          "message": "Error connecting to database"
        },
        "level": "fatal",
        "from": "app"
      },
      "source": "demo"
    }
  ]
}
```
So the output will look something like this:

```json
{
  "data": [
    "fatal",
    "warn"
  ]
}
```

Katchi connects to your pathivu server instance for live log tailing as well as viewing a log snapshot. It can be triggered in the following way:

```sh
$ katchi logs --host=http://localhost:5180
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:06:58 +0530 IST, line: INFO: == Kubernetes addon reconcile completed at 2019-11-17T18:36:58+00:00 ==
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:02 +0530 IST, line: INFO: Leader election disabled.
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:03 +0530 IST, line: INFO: == Kubernetes addon ensure completed at 2019-11-17T18:37:03+00:00 ==
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:03 +0530 IST, line: INFO: == Reconciling with deprecated label ==
 
APP: kube-addon-manager-minikube, ts: 2019-11-18 00:07:03 +0530 IST, line: error: no objects passed to apply
```

Learn more about Katchi [here](https://docs.pathivu.io/#/katchi)

<br>
<br>

## Usage

Pathivu can be deployed to your own kubernetes cluster, just follow the steps mentioned below.

```sh
# Create a namespace
kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/debug/namespace.yaml


# Create pathivu service
kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/debug/service.yaml

# Create pathivu deployment
kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/debug/deployment.yaml

```
Pathivu has an internal fluentd connector that can be used for log ingestion. The following command initialized the connector and starts shipping your service logs to pathivu.

```
kubectl create -f https://raw.githubusercontent.com/pathivu/pathivu/master/kubernetes/debug/fluentd.yaml
```
<br>
<br>

