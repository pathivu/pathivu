<img src="docs/img/pathivu.jpg" height="200" width="200" style="display: block;
  margin-left: auto;
  margin-right: auto;
  width: 50%;">    

# Pathivu: Logs you can search
Pathivu is a log aggreation system. It does all the indexing. All you have to shoot the query to find why account tranfer is failed.

It is primarly aimed for kubernetes but you can run on any deployment.

Not for production use yet. But, you can play around it.

## Usage
First run pathivu in your cluster.

```
kubectl create -f https://raw.githubusercontent.com/balajijinnah/chola/master/kubernetes/namespace.yaml

kubectl create -f https://raw.githubusercontent.com/balajijinnah/chola/master/kubernetes/deployment.yaml

kubectl create -f https://raw.githubusercontent.com/balajijinnah/chola/master/kubernetes/service.yaml
```
Then run pathivu fluentd plugin to ship logs to pathivu
```
kubectl create -f https://raw.githubusercontent.com/balajijinnah/chola/master/kubernetes/chola.yaml
```

Then run pathivu fluentd to ship logs to 
## Roadmap
#
- Multi node support.
- log shipper for several deploymets
- crash replayer.
- Feel free to GitHub issue for feature request.