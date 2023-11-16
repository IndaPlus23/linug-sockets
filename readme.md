# Tic tac toe multiplayer server

## Features

### Login

Login using usernames when connecting with the server

### Global Chat

When not in a game, all users are connected to a global chat. Messages are marked with the name of the sender

### Direct Messages

```zsh
dm <username> <message>
```


### Tic Tac Toe

#### Challenges

```zsh
challenge <username>
```

#### Accepting challenges

```zsh
accept <username>
```

After accepting a challenge a game of Tic Tac Toe will start

#### Resign

```zsh
resign
```

#### Recovering unfinished games

If a user is disconnected during a game, the game will automatically be recovered when reconnecting

