# SubAuction

SubAction is a project which brings various types of auctions of NFT tokens to Kusama and Polkadot. This project will allow users to
create auctions, add items represented by the NFT tokens, select the proper auction type and let
everybody compete and bid for the items of your choice.

!["Auction
Image"](https://img.etimg.com/thumb/width-600,height-400,imgsize-113275,resizemode-1,msid-76076103/news/politics-and-nation/auction-of-specially-plucked-teas-on-june-22-to-commemorate-international-tea-day.jpg)

## Design philosophy

Aim of this project is to create a platform which will support different kinds of auction for the upcoming prime time of the
NFT tokens. As in the real world, items represented by the NFTs can be bought, sold, transferred and even auctioned to
the highest bidder. There are actually various auction types serving multiple purposes and SubAuction's goal is to bring user one
unified and easy-to-use interface for both buyers and sellers. 
The whole process is configurable - user can set how long the auction should last, how many items will be auctioned
and other settings which can even differ by the auction type.
Also, very often users want to have multiple auctions which are started one by one. We would provide a framework which can
automate the process so user could actually create an auction template and then use it across its own defined sequence of auctions.

## Auction types 

MVP of this product will provide one single auction backend which will be the *english auction*, the classic between amongst auctions. But there are also very interesting auction types out there which could prove useful in the
blockchain space so here is a list of auctions which we find particularly suitable

### English auction 

The most common type of auction is where the participants bid against each other and the highest offer wins. Auction will
reach its end when one of the criterias are met

 - there is a countdown and when the auction is concluded, the highest bidder wins 
 - auction can have a time period between the bids. So for example if nobody made a better bid than the last one in 10
   minutes, auction is concluded and the last bidder wins 
 - auctioneer can even end the auction in any moment he chooses (usually when he is satisfied with the price)

There can be also various configurations parameters like auction length, minimum value of the next bid or list of invited
participants (so called "permissioned auction"). 

### Candle auction 

Candle auction is a variant of the English auction where the last part of the auction is randomized in the sense that it
can be concluded anytime. This prevents bidders from leaving their bids to the last few minutes or seconds and it's
actually more fair in the internet environment where bots can bid on behalf the buyers. Configuration of such an auction
will be very similar to the English one. 


### Dutch auction 

It's the opposite of the English one where auctioneer sets a rather high price and it keeps lowering it until one of the
bidders accept it. The price is always tied to the basket if items rather than a single one so price actually keeps
dropping until all of the items are sold. Auction is concluded when all the items are sold or the reserve price has been
reached. 

### Sealed first-price auction (blind auction)

In this scenario bidders will simultaneously submit their (secret) bids and winner is automatically selected by the
highest bid. The value of all the bids remains a secret and only auctioneer knows how much each participant have bid. This
type of auction is frequently used for auctioning the goverment contracts or mining licences. 


### Top-Up 

This is a very popular auction type used by charities. Each participant will pay a "top-up fee" which is based on the
bid he made minus the closest lower bid to his one. Which effectively means that each participant will pay the top-up
fee except the first and the last bidder. The last bidder wins the auction, obtains the item and only pays the item's
price.  

### Combinatorical auction 

There are mutliple variants of this kind of auctions but all of them revolve around the idea that multiple items are
being auctioned at the same time. So, for example, a bidder can say he will pay for A and B but only if he can get both
at the same time. 

## Implementation of the NFT token
You can find more information about our implementation and JSON metadata NFT format in our [docs](nft.md).

### NFT token pallet

We loosely based our `nft-pallet` on the [orml-nft](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/nft) implementation. We added couple of things to our own pallet
- Token locking (token owner shouldn't be able to transfer the NFT token after the auction is created)
- Token genesis for easier development
- Metadata followed by the ERC-721 [standard](https://docs.opensea.io/docs/metadata-standards)

Goal of our nft pallet is to allow our application to easily replace one NFT implementation for another.

### Auction pallet

We have developed our auction pallet with flexibility in mind. This is the list of features we currently support

 - Creating an auction with NFT token
 - Bidding by other users and locking their funds via the `LockableCurrency`
 - Auction time measured in blocks
 - Conclusion automated via `on_initialize` callback
 - Auction removal
 - Various checks to prevent malicious actions
 - Configuration parameters for the auction itself

## Developer instructions

### Build

The `make run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
make build
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/subauction -h
```

## Run

The `make run` command will launch a temporary node and its state will be discarded after you
terminate the process. After the project has been built, there are other ways to launch the node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/subauction --dev
```

Purge the development chain's state:

```bash
./target/release/subaction purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/node-template -lruntime=debug --dev
```

## Pallets 

### NFT tokens pallet 

### Auction pallet