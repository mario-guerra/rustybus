# rustybus
Demo app that uses all APIs in Azure Service Bus crate to send/receive/peek at messages in a named queue.

Setup:
From project root, modify .env file to point to your service bus implementation. You will need the name,
policy name, and policy key of your service bus instance, which can all be found in the Azure Portal in
your service bus dashboard. The plicy name and key can be found under Settings -> Shared Access Policies
in your service bus dashboard. 

Build:
From the root directory, run 'cargo build' to build the project. 

Usage:
There are several preconfigured JSON formatted messages available in the json_files directory, which can 
be used with the rust-listener project (https://github.com/mario-guerra/rust-listener). Messages intended
for sending to a queue must be piped as input to the rustybus executable.

Use one of the following commands to interact with the service bus:
To send:
>type .\json_files\sample_message_earth.json | cargo run send <queue name>
or
>type .\json_files\sample_message_earth.json | .\target\debug\rustybus.exe send <queue name>

To receive:
>cargo run receive demo
or
>.\target\debug\rustybus.exe receive demo

To peek:
>cargo run peek demo
or
>.\target\debug\rustybus.exe peek demo
