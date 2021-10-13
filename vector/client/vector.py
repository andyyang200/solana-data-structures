from account_info import Client
from solana.publickey import PublicKey
import struct

from solana.rpc.api import Client
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.transaction import Transaction, TransactionInstruction, AccountMeta
from solana.sysvar import SYSVAR_RENT_PUBKEY
from solana.system_program import SYS_PROGRAM_ID


PID = PublicKey('9ijP4o3M3PoZ57oP3Jkyjq54ZU3vt2JSnAvvomfU6woF')

class Vector:
    
    def __init__(self, auth, max_length=1048576, element_size=1, num_accounts = 10, program_id=PID, run_transaction = True):
        assert(isinstance(auth, Keypair))

        self.solana_client = Client("https://api.devnet.solana.com")

        self.auth = auth
        self.element_size = element_size
        self.max_length = max_length
        self.num_accounts = num_accounts
        self.program_id = program_id if isinstance(program_id, PublicKey) else PublicKey(program_id)
        self.meta_key, self.meta_bumper = PublicKey.find_program_address([bytes(auth.public_key), struct.pack('<Q', max_length), struct.pack('<Q', element_size)], self.program_id)
        
        self.account_keys = []
        self.account_bumpers = []
        for i in range(0, num_accounts):
            key, bumper = PublicKey.find_program_address([bytes(self.meta_key), struct.pack('<B', i)], self.program_id)
            self.account_keys.append(key)
            self.account_bumpers.append(bumper)

        if not run_transaction:
            return

        keys = [
            AccountMeta(self.auth.public_key, True, False),
            AccountMeta(self.meta_key, False, True),
            AccountMeta(SYS_PROGRAM_ID, False, False),
            AccountMeta(SYSVAR_RENT_PUBKEY, False, False),
        ]
        for i in range(0, num_accounts):
            keys += [AccountMeta(self.account_keys[i], False, True)]

        instruction_data = struct.pack('<BQQB'+'B'*num_accounts, 0, max_length, element_size, self.meta_bumper, *self.account_bumpers)
        instruction = TransactionInstruction(keys, program_id, instruction_data)

        tx = Transaction().add(instruction)
        self.init_tx_sig = self.solana_client.send_transaction(tx, auth)

    def push(self, data):

        keys = [
            AccountMeta(self.meta_key, False, True),
        ]
        for i in range(0, self.num_accounts):
            keys += [AccountMeta(self.account_keys[i], False, True)]

        instruction_data = struct.pack('<B', 1) + data
        instruction = TransactionInstruction(keys, self.program_id, instruction_data)

        tx = Transaction().add(instruction)
        tx_sig = self.solana_client.send_transaction(tx, self.auth)
        return tx_sig

    def pop(self, num_elements):

        keys = [
            AccountMeta(self.meta_key, False, True),
        ]
        for i in range(0, self.num_accounts):
            keys += [AccountMeta(self.account_keys[i], False, True)]

        instruction_data = struct.pack('<BQ', 2, num_elements)
        instruction = TransactionInstruction(keys, self.program_id, instruction_data)

        tx = Transaction().add(instruction)
        tx_sig = self.solana_client.send_transaction(tx, self.auth)
        return tx_sig

    def get(self, start, end):
        keys = [
            AccountMeta(self.meta_key, False, True),
        ]
        for i in range(0, self.num_accounts):
            keys += [AccountMeta(self.account_keys[i], False, True)]

        instruction_data = struct.pack('<BQQ', 3, start, end)
        instruction = TransactionInstruction(keys, self.program_id, instruction_data)

        tx = Transaction().add(instruction)
        tx_sig = self.solana_client.send_transaction(tx, self.auth)
        return tx_sig

    def delete(self):
        keys = [
            AccountMeta(self.auth.public_key, True, False),
            AccountMeta(self.meta_key, False, True),
        ]
        for i in range(0, self.num_accounts):
            keys += [AccountMeta(self.account_keys[i], False, True)]

        instruction_data = struct.pack('<B', 4)
        instruction = TransactionInstruction(keys, self.program_id, instruction_data)

        tx = Transaction().add(instruction)
        tx_sig = self.solana_client.send_transaction(tx, self.auth)
        return tx_sig





        
    