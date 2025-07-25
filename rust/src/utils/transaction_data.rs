#[derive(Debug, Clone, Default)]
pub struct TransactionData {
    pub transaction_id: String,
    pub miner_input_address: String,
    pub miner_input_amount: String,
    pub trader_output_address: String,
    pub trader_output_amount: String,
    pub miner_change_address: String,
    pub miner_change_amount: String,
    pub transaction_fees: String,
    pub block_height: String,
    pub block_hash: String,
}

impl TransactionData {
    // Constructeur vide
    pub fn new() -> Self {
        TransactionData::default()
    }
    
    // Constructeur avec tous les paramètres
    pub fn with_all_fields(
        transaction_id: String,
        miner_input_address: String,
        miner_input_amount: String,
        trader_output_address: String,
        trader_output_amount: String,
        miner_change_address: String,
        miner_change_amount: String,
        transaction_fees: String,
        block_height: String,
        block_hash: String,
    ) -> Self {
        TransactionData {
            transaction_id,
            miner_input_address,
            miner_input_amount,
            trader_output_address,
            trader_output_amount,
            miner_change_address,
            miner_change_amount,
            transaction_fees,
            block_height,
            block_hash,
        }
    }
    
    // Méthodes pour définir chaque champ individuellement
    pub fn set_transaction_id(&mut self, transaction_id: String) {
        self.transaction_id = transaction_id;
    }
    
    pub fn set_miner_input_address(&mut self, address: String) {
        self.miner_input_address = address;
    }
    
    pub fn set_miner_input_amount(&mut self, amount: String) {
        self.miner_input_amount = amount;
    }
    
    pub fn set_trader_output_address(&mut self, address: String) {
        self.trader_output_address = address;
    }
    
    pub fn set_trader_output_amount(&mut self, amount: String) {
        self.trader_output_amount = amount;
    }
    
    pub fn set_miner_change_address(&mut self, address: String) {
        self.miner_change_address = address;
    }
    
    pub fn set_miner_change_amount(&mut self, amount: String) {
        self.miner_change_amount = amount;
    }
    
    pub fn set_transaction_fees(&mut self, fees: String) {
        self.transaction_fees = fees;
    }
    
    pub fn set_block_height(&mut self, height: String) {
        self.block_height = height;
    }
    
    pub fn set_block_hash(&mut self, hash: String) {
        self.block_hash = hash;
    }
    
    // Méthode pour convertir en vecteur dans l'ordre requis
    pub fn to_ordered_vec(&self) -> Vec<String> {
        vec![
            self.transaction_id.clone(),
            self.miner_input_address.clone(),
            self.miner_input_amount.clone(),
            self.trader_output_address.clone(),
            self.trader_output_amount.clone(),
            self.miner_change_address.clone(),
            self.miner_change_amount.clone(),
            self.transaction_fees.clone(),
            self.block_height.clone(),
            self.block_hash.clone(),
        ]
    }
    
    // Méthode pour vérifier si tous les champs sont remplis
    pub fn is_complete(&self) -> bool {
        !self.transaction_id.is_empty() &&
        !self.miner_input_address.is_empty() &&
        !self.miner_input_amount.is_empty() &&
        !self.trader_output_address.is_empty() &&
        !self.trader_output_amount.is_empty() &&
        !self.miner_change_address.is_empty() &&
        !self.miner_change_amount.is_empty() &&
        !self.transaction_fees.is_empty() &&
        !self.block_height.is_empty() &&
        !self.block_hash.is_empty()
    }
    
    // Méthode pour obtenir les noms des champs dans l'ordre
    pub fn field_names() -> Vec<&'static str> {
        vec![
            "transaction_id",
            "miner_input_address", 
            "miner_input_amount",
            "trader_output_address",
            "trader_output_amount",
            "miner_change_address",
            "miner_change_amount", 
            "transaction_fees",
            "block_height",
            "block_hash"
        ]
    }
}
