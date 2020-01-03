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


this_dir = File.expand_path(File.dirname(__FILE__))
lib_dir = File.join(File.dirname(this_dir), 'lib')
$LOAD_PATH.unshift(lib_dir) unless $LOAD_PATH.include?(lib_dir)

require 'fluent/plugin/output'
require 'uea-stemmer'
require 'stopwords'
require 'yajl'
require 'net/http'
require_relative 'api_services_pb'
require 'grpc'

include Api
module Fluent::Plugin
  class CholaOutput < Output
    Fluent::Plugin.register_output('chola', self)
    helpers :inject, :formatter, :compat_parameters
    @@client = nil
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

    def flatten_hash(hash)
      hash.each_with_object({}) do |(k, v), h|
        if v.is_a? Hash
          flatten_hash(v).map do |h_k, h_v|
            h["#{k}.#{h_k}".to_sym] = h_v
          end
        else 
          h[k] = v
        end
      end
    end
  
   def build_indexes(flattened_data)
      stemmer = UEAStemmer.new
      indexes = []
      flattened_data.each do|key,value|
          if value.kind_of?(Array)
              value.each do|inner|
                  splits = inner.gsub(/\s+/m, ' ').strip.split(" ")
                  splits.each do|item|
                    if !Stopwords.is?(item)
                      indexes.push(stemmer.stem(item))
                    end
                  end
              end  
          else
            splits = value.gsub(/\s+/m, ' ').strip.split(" ")
            splits.each do|item|
              if !Stopwords.is?(item)
                indexes.push(stemmer.stem(item))
              end
            end
          end    
        end
      return indexes
    end

    def write(chunk)
        if @@client == nil
          @@client = Api::Pathivu::Stub.new(url, :this_channel_is_insecure)
        end

        partitions = {}
        chunk.each do |time, record|
            unless record.is_a?(Hash)
              @log.warn 'Dropping log entries with malformed record: ' \
                        "'#{record.inspect}' from tag '#{tag}' at '#{time}'. " \
                        'A log record should be in JSON format.'
              next
            end
            flattened_hash = self.flatten_hash(record)
            indexes = self.build_indexes(flattened_hash)
            line = PushLogLine::new(ts: Time.at(time.to_f).to_i, indexes: indexes, structured: true, json_keys: flattened_hash.keys, raw_data: Yajl.dump(record))
            if !partitions.key?(record["kubernetes"]["pod_name"])
              partitions[record["kubernetes"]["pod_name"]] = []
            end
            partitions[record["kubernetes"]["pod_name"]].push(line)
        end
        #post each parition
        partitions.each do|partition, lines|
          req = PushRequest::new(source: partition, lines: lines)
          @@client.push(req)
        end
    end
  end
end
