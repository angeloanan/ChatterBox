# ChatterBox
💬 | A microservice to handle chat / guest book for my personal website!


* [X] Establish a WS connection
* [X] Broadcast sent message to everyone
  * [ ] BUG: A message is sent when someone left
* [ ] Integrate a standardized broadcast format for message: Username, Message, Timestamp
* [X] Store message to database
  * [X] Initialize database
  * [ ] Periodic clean up old messages (older than 1 week)
* [ ] Send message history on ws connection
* [ ] Able to delete message for moderation purposes
* [ ] Assign default username
  * [ ] Store username per IP
* [ ] Commands (/help, /nick)
* [ ] Connect to discord
  * [ ] Forward message to my DMs / private channel
  * [ ] Send messages as myself to the chatbox from Discord