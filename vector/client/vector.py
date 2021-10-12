from .account_info import Client
from solana.publickey import PublicKey

PID = PublicKey('88uZSCS75MubFukG9XsAY4uaTNDuHexLQe7B6mQexx5d')

class Vector:
    
    def __init__(self, element_size=1, max_length=1048576, seeds=[[b'meta'], [b'vector']], program_id=PID):
        self.element_size = element_size
        self.max_length = max_length
        self.program_id = program_id if isinstance(program_id, PublicKey) else PublicKey(program_id)
        self.meta_key, self.meta_bumber = PublicKey.find_program_address(seeds[0], self.program_id)
        
        self.account_keys = []
        self.account_bumpers = []
        for i in range(1, len(seeds)):
            key, bumper = PublicKey.find_program_address(seeds[i], self.program_id)
            self.account_keys.append(key)
            self.account_bumpers.append(bumper)
    