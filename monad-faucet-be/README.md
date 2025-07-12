# Faucet for the Monad chain

- send ERC20
- send ETH/MONAD
- deploy token contract (80% to us and 20% to the user)
- maintain a list of tokens
- github authentication and store usernames
- store history of claims
- token image upload to imgix

- Github authentication
  - signin in fe
  - redirect to github
  - get the code
  - post the code to github to get the token
  - post the key and get the jwt
  - store the jwt in the cookie
  - how to know if the user is logged in?
