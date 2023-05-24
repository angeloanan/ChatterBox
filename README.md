# ChatterBox
💬 | A microservice to handle chat / guest book for my personal website!


* [X] Establish a WS connection
* [X] Broadcast sent message to everyone
  * [X] BUG: A message is sent when someone left
* [X] Integrate a standardized broadcast format for message: `TIME USERNAME: MESSAGE`
* [X] Store message to database
  * [X] Initialize database
  * [X] Periodic clean up old messages (older than 1 week)
* [X] Send message history on ws connection
* [ ] Able to delete message for moderation purposes
* [X] Assign default username
  * [ ] Store username per IP
    * [X] DB Schema & fns
    * [ ] Convert IP to Hex
* [ ] Commands (/help, /nick)
* [ ] Connect to discord
  * [ ] Forward message to my DMs / private channel
  * [ ] Send messages as myself to the chatbox from Discord
