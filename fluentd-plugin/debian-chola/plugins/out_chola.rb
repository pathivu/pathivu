#
# Fluentd
#
#    Licensed under the Apache License, Version 2.0 (the "License");
#    you may not use this file except in compliance with the License.
#    You may obtain a copy of the License at
#
#        http://www.apache.org/licenses/LICENSE-2.0
#
#    Unless required by applicable law or agreed to in writing, software
#    distributed under the License is distributed on an "AS IS" BASIS,
#    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#    See the License for the specific language governing permissions and
#    limitations under the License.
#
# 
#   Copyright 2019 Balaji Jinnah and Contributors
#  
#   Licensed under the Apache License, Version 2.0 (the "License");
#   you may not use this file except in compliance with the License.
#   You may obtain a copy of the License at
#  
#       http://www.apache.org/licenses/LICENSE-2.0
#  
#   Unless required by applicable law or agreed to in writing, software
#   distributed under the License is distributed on an "AS IS" BASIS,
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#   See the License for the specific language governing permissions and
#   limitations under the License.
#  

require 'fluent/plugin/output'
require 'uea-stemmer'
require 'stopwords'
require 'yajl'
require 'net/http'

module Fluent::Plugin
  class CholaOutput < Output
    Fluent::Plugin.register_output('chola', self)

    helpers :inject, :formatter, :compat_parameters

    DEFAULT_LINE_FORMAT_TYPE = 'stdout'
    DEFAULT_FORMAT_TYPE = 'json'

    desc 'url of chola server'
    config_param :url, :string

    config_section :buffer do
      config_set_default :chunk_keys, ['tag']
      config_set_default :flush_at_shutdown, true
      config_set_default :chunk_limit_size, 10 * 1024
    end

    config_section :format do
      config_set_default :@type, DEFAULT_LINE_FORMAT_TYPE
      config_set_default :output_type, DEFAULT_FORMAT_TYPE
    end

    def prefer_buffered_processing
      false
    end

    def multi_workers_ready?
      false
    end

    def write(chunk)
        stemmer = UEAStemmer.new
        partitions = {}
        chunk.each do |time, record|
            unless record.is_a?(Hash)
              @log.warn 'Dropping log entries with malformed record: ' \
                        "'#{record.inspect}' from tag '#{tag}' at '#{time}'. " \
                        'A log record should be in JSON format.'
              next
            end
            splits = record["log"].gsub(/\s+/m, ' ').strip.split(" ")
            indexes = []
            splits.each do|item|
              if !Stopwords.is?(item)
                indexes.push(stemmer.stem(item))
              end
            end
            if !partitions.key?(record["kubernetes"]["pod_name"])
              partitions[record["kubernetes"]["pod_name"]] = []
            end
            partitions[record["kubernetes"]["pod_name"]].push({"line": record["log"], "ts": Time.at(time.to_f).to_i, "indexes":indexes})
        end
        #post each parition
        partitions.each do|parition, entries|
          uri = URI.parse(url + '/push')
          req = Net::HTTP::Post.new(
            uri.request_uri
          )
          req.add_field('Content-Type', 'application/json')
          data = { "app" => parition,"lines" => entries}
          req.body = Yajl.dump(data)
          res = Net::HTTP.start(uri.hostname, uri.port) { |http| http.request(req) }
          unless res&.is_a?(Net::HTTPSuccess)
          res_summary = if res
                          "#{res.code} #{res.message} #{res.body}"
                        else
                          'res=nil'
                        end
          log.warn "failed to #{req.method} #{uri} (#{res_summary})"
          log.warn Yajl.dump(data)
          end
        end

    end
  end
end
