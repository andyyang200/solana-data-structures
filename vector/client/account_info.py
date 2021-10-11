from solana.rpc.api import Client as SolanaClient
from solana.publickey import PublicKey
from base64 import b64decode
from base58 import b58decode

DECODER = {'base58': b58decode, 'base64': b64decode}

class Client:
    
    def __init__(self, endpoint='https://api.devnet.solana.com', **kwargs):
        self.solana_client = SolanaClient(endpoint=endpoint, **kwargs)

    def account_info(self, pubkey):
        if not isinstance(pubkey, PublicKey):
            pubkey = PublicKey(pubkey)
        return self.solana_client.get_account_info(pubkey).get('result', {}).get('value')

    def account_data(self, pubkey):
        if not isinstance(pubkey, PublicKey):
            pubkey = PublicKey(pubkey)
        data = self.solana_client.get_account_info(pubkey).get('result', {}).get('value', {}).get('data')
        if data is None:
            return
        elif data[1] not in DECODER:
            raise RuntimeError(f'{data[1]} is not a recognized encoding!')
        return DECODER[data[1]](data[0])

