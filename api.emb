/// username or uuid of a user
type User <string>

/// gets the username history of a user
api profiles names <user: User>
  | api profile {user}
  | http api.mojang.com/user/profiles/{_.id}/names
  | json [ { name: string }, ...{ name: string, changedToAt: int } ]

/// gets the profile of user
api profiles <user: User>
  | http api.mojang.com/user/profiles/{user}
  | json { id: string, username: string }