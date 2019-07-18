// This module demonstrates simple variant of remoting usage. He contains two submodule - client
// and server. In the server deploys the Network ActorSystem and run Storage actor in her. This
// actor may receive GetData message and respond with Data message. In the client deploys
// LocalActorSystem and RemoteActorSystem. The last system is point to the server's actor system.
// Requestor actor periodically sends GetData message to the Storage actor deployed on the server
// actor system and receives Data message as response.

pub mod client;
pub mod server;