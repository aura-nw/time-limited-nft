# time-limited-nft
The type of nft has an expiry date

## How to use scripts
### 1. Install dependencies
```console
cd scripts
npm install
```

### 2. Deploy contract
```console
node ./scripts/0_contracts_setup.js
```

### 3. Mint NFT
#### 3.1. Modify information of new NFT at `./scripts/1_mint_nft.js`:
| Line No. | Variable             | Description                                            |
| -------- | -------------------- | ------------------------------------------------------ |
| 17       | `COLLECTION_ADDRESS` | The address of NFT collection after deploying contract |
| 133      | `token_id`           | The id of NFT in the collection                        |
| 134      | `owner`              | The owner of NFT                                       |
| 137      | `image`              | The image of NFT                                       |
| 138      | `image_data`         | The image data of NFT                                  |
| 140      | `description`        | The description of NFT                                 |
| 149      | `at_time`            | The timestamp of expiry time                           |

#### 3.2. Mint NFT
```console
node ./scripts/1_mint_nft.js
```

### 4. Burn NFT
#### 4.1. Modify information of NFT at `./scripts/2_burn_nft.js`:
| Line No. | Variable   | Description                          |
| -------- | ---------- | ------------------------------------ |
| 133      | `token_id` | The id of NFT that user want to burn |

#### 4.2. Burn NFT
```console
node ./scripts/2_burn_nft.js
```