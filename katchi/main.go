package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"net/url"
	"os"
	"path"
	"time"

	"github.com/spf13/cobra"
)

type client struct {
	host string
}

// PartitionRes is reponse struct for partition
type PartitionRes struct {
	Partitions []string `json:"partitions"`
}

// QueryReq to collect logs
type QueryReq struct {
	Query      string   `json:"query"`
	StartTs    int64    `json:"start_ts"`
	EndTs      int64    `json:"end_ts"`
	Count      int      `json:"count"`
	Offset     int      `json:"offset"`
	Partitions []string `json:"partitions"`
}

// Line is log line
type Line struct {
	Line string `json:"line"`
	Ts   int64  `json:"ts"`
	App  string `json:"app"`
}

// QueryRes log query response
type QueryRes struct {
	Lines []Line `json:"lines"`
}

func (c *client) partitions() []string {
	url, err := url.Parse(host)
	if err != nil {
		panic(err)
	}
	url.Path = path.Join(url.Path, "partitions")
	resp, err := http.Get(url.String())
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	res := &PartitionRes{}
	if err = json.Unmarshal(body, res); err != nil {
		panic(err)
	}
	return res.Partitions
}

func (c *client) query(reqBody *QueryReq) QueryRes {
	if reqBody.Partitions == nil {
		reqBody.Partitions = []string{}
	}
	url, err := url.Parse(host)
	if err != nil {
		panic(err)
	}
	url.Path = path.Join(url.Path, "query")

	body, err := json.Marshal(reqBody)
	if err != nil {
		log.Fatal(err)
	}
	req, err := http.NewRequest("POST", url.String(), bytes.NewBuffer(body))
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	resBody, _ := ioutil.ReadAll(resp.Body)
	res := QueryRes{}
	if err = json.Unmarshal(resBody, &res); err != nil {
		panic(err)
	}
	return res
}

var host string

var req *QueryReq

var since time.Duration

var partitions []string

var startTs int64

var endTs int64

func main() {
	req = new(QueryReq)
	since = (time.Second * 0)
	rootCmd := &cobra.Command{
		Use:   "katchi",
		Short: "Katchi is log visualization tool for pathivu",
		Long:  `katch is a cli tool for pathivu, which let's you folks to view all the logs in pathivu`,
	}
	var queryCmd = &cobra.Command{
		Use:   "logs --partitions=kube-server,api-server --since=3h --query=info",
		Short: "query logs ",
		Long:  `query logs in the pathivu server`,
		Run: func(cmd *cobra.Command, args []string) {
			if host == "" {
				log.Fatalf("host name required. Set pathivu_HOST env or pass host flag")
			}
			c := &client{
				host: host,
			}
			req.Partitions = partitions
			if since.Seconds() != 0 {
				req.StartTs = time.Now().Add(-since).Unix()
				req.EndTs = time.Now().Unix()
			}
			res := c.query(req)
			for _, line := range res.Lines {
				fmt.Printf("APP: %s, ts: %s, line: %s \n", line.App, time.Unix(line.Ts, 0).String(), line.Line)
			}
		},
	}
	queryCmd.Flags().DurationVar(&since, "since", time.Second*0, "since=1h")
	queryCmd.Flags().StringArrayVar(&partitions, "apps", []string{}, "apps=kubeserver,api-server")

	partitionCmd := &cobra.Command{
		Use:   "apps",
		Short: "apps gives all the app name of logs that has been ingested in the pathivu",
		Long:  `apps gives all the app name of logs that has been ingested in the pathivu`,
		Run: func(cmd *cobra.Command, args []string) {
			if host == "" {
				log.Fatalf("katch is a cli tool for pathivu, which let's you folks to view all the logs in pathivuhost name required. Set pathivu_HOST env or pass host flag")
			}
			c := &client{
				host: host,
			}
			partitons := c.partitions()
			fmt.Println("App Names")
			for _, app := range partitons {
				fmt.Println(app)
			}
		},
	}
	rootCmd.PersistentFlags().StringVar(&host, "host", os.Getenv("pathivu_HOST"), "pathivu host address")
	rootCmd.AddCommand(queryCmd)
	rootCmd.AddCommand(partitionCmd)
	rootCmd.Execute()
}
