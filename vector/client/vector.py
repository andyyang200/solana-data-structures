from .account_info import Client
from solana.publickey import PublicKey
import struct

from solana.rpc.api import Client
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.transaction import Transaction, TransactionInstruction, AccountMeta
from solana.sysvar import SYSVAR_RENT_PUBKEY
from solana.system_program import SYS_PROGRAM_ID


PID = PublicKey('88uZSCS75MubFukG9XsAY4uaTNDuHexLQe7B6mQexx5d')

class Vector:
    
    def __init__(self, auth, max_length=1048576, element_size=1, num_accounts = 10, program_id=PID):
        assert(isinstance(auth, Keypair))
        self.element_size = element_size
        self.max_length = max_length
        self.program_id = program_id if isinstance(program_id, PublicKey) else PublicKey(program_id)
        self.meta_key, self.meta_bumper = PublicKey.find_program_address([bytes(auth.public_key), struct.pack('<Q', max_length), struct.pack('<Q', element_size)], self.program_id)
        
        self.account_keys = []
        self.account_bumpers = []
        for i in range(0, num_accounts):
            key, bumper = PublicKey.find_program_address([bytes(self.meta_key), struct.pack('<B', i)], self.program_id)
            self.account_keys.append(key)
            self.account_bumpers.append(bumper)

        keys = [
            AccountMeta(auth.key, True, False),
            AccountMeta(self.meta_key, False, True),
            AccountMeta(SYSVAR_RENT_PUBKEY, False, False),
        ]
        for i in range(0, num_accounts):
            keys += [AccountMeta(self.account_keys[i], False, True)]

        instruction_data = struct.pack('<BQQB'+'B'*num_accounts, 0, max_length, element_size)
        instruction = TransactionInstruction{
            keys, program_id, 

        }

        
    