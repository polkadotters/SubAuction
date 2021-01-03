NFT representation and metadata standard
=====
NFT tokens are created according to the ERC-721 standard with the help of the [orml-nft](https://github.com/open-web3-stack/open-runtime-module-library/tree/0.3.2/nft) pallet. All tokens also contain metadata about it's underlying asset and we chose to follow the OpenSea metadata [standard](https://docs.opensea.io/docs/metadata-standards).

Each token also needs to belong to the token class which represents a basket of items so every single NFT token also contain `class_id`.

Here is a simple JSON describing how such a basket of items could look like

```json
{
  "classes":  [{
    "name": "Paintings",
    "description": "A token class that represents paintings",
    "tokens": [
      {
        "name": "Bleeee",
        "description": "Desc",
        "external_url": "none yet",
        "image": "none yet"
      },
      {
        "name": "Bloody",
        "description": "Desc",
        "external_url": "none yet",
        "image": "none yet"
      },
      {
        "name": "psycho",
        "description": "Desc",
        "external_url": "none yet",
        "image": "none yet"
      }
    ]
  }]
}
```

Here is a single class which can be fed into the Polkadot JS API while calling the extrinsic

```json
{
  "name": "Paintings",
  "description":"A token class that represents paintings"
}
```

And here is a single NFT token which are going to mint
```json
{
  "name": "psycho",
  "description": "Desc",
  "external_url": "none yet",
  "image": "none yet"
}

```

So first go to the Polkadot JS -> Developer -> Extrinsics and choose `nft` pallet with `createClass`. Submit following string as metadata 
```json
{"name":"psycho","description":"Desc","external_url":"none yet","image":"none yet"}
```
 and leave the `data` field empty.

Then choose the `createToken` extrinsic and submit following metadata for your token
```json
{"name":"psycho","description":"Desc","external_url":"none yet","image":"none yet"}
```
with leaving the `token_data` empty as well.

Now go to the Developer -> Chain state, choose `ormlNft` and query your classes or tokens with their respective ids.

