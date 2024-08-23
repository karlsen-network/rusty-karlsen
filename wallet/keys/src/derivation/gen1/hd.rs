use crate::derivation::traits::*;
use crate::imports::*;
use hmac::Mac;
use karlsen_addresses::{Address, Prefix as AddressPrefix, Version as AddressVersion};
use karlsen_bip32::types::{ChainCode, HmacSha512, KeyFingerprint, PublicKeyBytes, KEY_SIZE};
use karlsen_bip32::{
    AddressType, ChildNumber, DerivationPath, ExtendedKey, ExtendedKeyAttrs, ExtendedPrivateKey,
    ExtendedPublicKey, Prefix, PrivateKey, PublicKey, SecretKey, SecretKeyExt,
};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::fmt::Debug;
// use wasm_bindgen::prelude::*;

fn get_fingerprint<K>(private_key: &K) -> KeyFingerprint
where
    K: PrivateKey,
{
    let public_key_bytes = private_key.public_key().to_bytes();

    let digest = Ripemd160::digest(Sha256::digest(public_key_bytes));
    digest[..4].try_into().expect("digest truncated")
}

#[derive(Clone)]
// #[wasm_bindgen(inspectable)]
pub struct PubkeyDerivationManager {
    /// Derived public key
    public_key: secp256k1::PublicKey,
    /// Extended key attributes.
    attrs: ExtendedKeyAttrs,
    #[allow(dead_code)]
    fingerprint: KeyFingerprint,
    hmac: HmacSha512,
    index: Arc<Mutex<u32>>,
}

impl PubkeyDerivationManager {
    pub fn new(
        public_key: secp256k1::PublicKey,
        attrs: ExtendedKeyAttrs,
        fingerprint: KeyFingerprint,
        hmac: HmacSha512,
        index: u32,
    ) -> Result<Self> {
        let wallet = Self {
            public_key,
            attrs,
            fingerprint,
            hmac,
            index: Arc::new(Mutex::new(index)),
        };

        Ok(wallet)
    }

    pub fn derive_pubkey_range(
        &self,
        indexes: std::ops::Range<u32>,
    ) -> Result<Vec<secp256k1::PublicKey>> {
        let list = indexes
            .map(|index| self.derive_pubkey(index))
            .collect::<Vec<_>>();
        let keys = list.into_iter().collect::<Result<Vec<_>>>()?;
        Ok(keys)
    }

    pub fn derive_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        let (key, _chain_code) = WalletDerivationManager::derive_public_key_child(
            &self.public_key,
            index,
            self.hmac.clone(),
        )?;
        Ok(key)
    }

    pub fn create_address(
        key: &secp256k1::PublicKey,
        prefix: AddressPrefix,
        ecdsa: bool,
    ) -> Result<Address> {
        let address = if ecdsa {
            let payload = &key.serialize();
            Address::new(prefix, AddressVersion::PubKeyECDSA, payload)
        } else {
            let payload = &key.x_only_public_key().0.serialize();
            Address::new(prefix, AddressVersion::PubKey, payload)
        };

        Ok(address)
    }

    pub fn public_key(&self) -> ExtendedPublicKey<secp256k1::PublicKey> {
        self.into()
    }

    pub fn attrs(&self) -> &ExtendedKeyAttrs {
        &self.attrs
    }

    /// Serialize the raw public key as a byte array.
    pub fn to_bytes(&self) -> PublicKeyBytes {
        self.public_key().to_bytes()
    }

    /// Serialize this key as an [`ExtendedKey`].
    pub fn to_extended_key(&self, prefix: Prefix) -> ExtendedKey {
        let mut key_bytes = [0u8; KEY_SIZE + 1];
        key_bytes[..].copy_from_slice(&self.to_bytes());
        ExtendedKey {
            prefix,
            attrs: self.attrs.clone(),
            key_bytes,
        }
    }

    pub fn to_string(&self) -> Zeroizing<String> {
        Zeroizing::new(self.to_extended_key(Prefix::KPUB).to_string())
    }
}

// #[wasm_bindgen]
impl PubkeyDerivationManager {
    // #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn get_public_key(&self) -> String {
        self.public_key().to_string(None)
    }
}

impl From<&PubkeyDerivationManager> for ExtendedPublicKey<secp256k1::PublicKey> {
    fn from(inner: &PubkeyDerivationManager) -> ExtendedPublicKey<secp256k1::PublicKey> {
        ExtendedPublicKey {
            public_key: inner.public_key,
            attrs: inner.attrs().clone(),
        }
    }
}

#[async_trait]
impl PubkeyDerivationManagerTrait for PubkeyDerivationManager {
    fn new_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.set_index(self.index()? + 1)?;
        self.current_pubkey()
    }

    fn index(&self) -> Result<u32> {
        Ok(*self.index.lock()?)
    }

    fn set_index(&self, index: u32) -> Result<()> {
        *self.index.lock()? = index;
        Ok(())
    }

    fn current_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let index = self.index()?;
        let key = self.derive_pubkey(index)?;

        Ok(key)
    }

    fn get_range(&self, range: std::ops::Range<u32>) -> Result<Vec<secp256k1::PublicKey>> {
        self.derive_pubkey_range(range)
    }
}

#[derive(Clone)]
pub struct WalletDerivationManager {
    /// extended public key derived upto `m/<Purpose>'/121337'/<Account Index>'`
    extended_public_key: ExtendedPublicKey<secp256k1::PublicKey>,

    /// receive address wallet
    receive_pubkey_manager: Arc<PubkeyDerivationManager>,

    /// change address wallet
    change_pubkey_manager: Arc<PubkeyDerivationManager>,
}

impl WalletDerivationManager {
    pub fn create_extended_key_from_xprv(
        xprv: &str,
        is_multisig: bool,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let xprv_key = ExtendedPrivateKey::<SecretKey>::from_str(xprv)?;
        Self::derive_extended_key_from_master_key(xprv_key, is_multisig, account_index)
    }

    pub fn derive_extended_key_from_master_key(
        xprv_key: ExtendedPrivateKey<SecretKey>,
        is_multisig: bool,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let attrs = xprv_key.attrs();

        let (extended_private_key, attrs) = Self::create_extended_key(
            *xprv_key.private_key(),
            attrs.clone(),
            is_multisig,
            account_index,
        )?;

        Ok((extended_private_key, attrs))
    }

    fn create_extended_key(
        mut private_key: SecretKey,
        mut attrs: ExtendedKeyAttrs,
        is_multisig: bool,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let purpose = if is_multisig { 45 } else { 44 };
        let address_path = format!("{purpose}'/121337'/{account_index}'");
        let children = address_path.split('/');
        for child in children {
            (private_key, attrs) =
                Self::derive_private_key(&private_key, &attrs, child.parse::<ChildNumber>()?)?;
        }

        Ok((private_key, attrs))
    }

    pub fn build_derivate_path(
        is_multisig: bool,
        account_index: u64,
        cosigner_index: Option<u32>,
        address_type: Option<AddressType>,
    ) -> Result<DerivationPath> {
        if is_multisig && cosigner_index.is_none() {
            return Err("cosigner_index is required for multisig path derivation"
                .to_string()
                .into());
        }
        let purpose = if is_multisig { 45 } else { 44 };
        let mut path = format!("m/{purpose}'/121337'/{account_index}'");
        if let Some(cosigner_index) = cosigner_index {
            path = format!("{path}/{}", cosigner_index)
        }
        if let Some(address_type) = address_type {
            path = format!("{path}/{}", address_type.index());
        }
        let path = path.parse::<DerivationPath>()?;
        Ok(path)
    }

    pub fn receive_pubkey_manager(&self) -> &PubkeyDerivationManager {
        &self.receive_pubkey_manager
    }
    pub fn change_pubkey_manager(&self) -> &PubkeyDerivationManager {
        &self.change_pubkey_manager
    }

    pub fn derive_child_pubkey_manager(
        mut public_key: ExtendedPublicKey<secp256k1::PublicKey>,
        address_type: AddressType,
        cosigner_index: Option<u32>,
    ) -> Result<PubkeyDerivationManager> {
        if let Some(cosigner_index) = cosigner_index {
            public_key = public_key.derive_child(ChildNumber::new(cosigner_index, false)?)?;
        }

        public_key = public_key.derive_child(ChildNumber::new(address_type.index(), false)?)?;

        let mut hmac = HmacSha512::new_from_slice(&public_key.attrs().chain_code)
            .map_err(karlsen_bip32::Error::Hmac)?;
        hmac.update(&public_key.to_bytes());

        PubkeyDerivationManager::new(
            *public_key.public_key(),
            public_key.attrs().clone(),
            public_key.fingerprint(),
            hmac,
            0,
        )
    }

    pub fn derive_public_key(
        public_key: &secp256k1::PublicKey,
        attrs: &ExtendedKeyAttrs,
        index: u32,
    ) -> Result<(secp256k1::PublicKey, ExtendedKeyAttrs)> {
        let fingerprint = public_key.fingerprint();

        let mut hmac =
            HmacSha512::new_from_slice(&attrs.chain_code).map_err(karlsen_bip32::Error::Hmac)?;
        hmac.update(&public_key.to_bytes());

        let (key, chain_code) = Self::derive_public_key_child(public_key, index, hmac)?;

        let depth = attrs
            .depth
            .checked_add(1)
            .ok_or(karlsen_bip32::Error::Depth)?;

        let attrs = ExtendedKeyAttrs {
            parent_fingerprint: fingerprint,
            child_number: ChildNumber::new(index, false)?,
            chain_code,
            depth,
        };

        Ok((key, attrs))
    }

    fn derive_public_key_child(
        key: &secp256k1::PublicKey,
        index: u32,
        mut hmac: HmacSha512,
    ) -> Result<(secp256k1::PublicKey, ChainCode)> {
        let child_number = ChildNumber::new(index, false)?;
        hmac.update(&child_number.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (child_key, chain_code) = result.split_at(KEY_SIZE);

        // We should technically loop here if a `secret_key` is zero or overflows
        // the order of the underlying elliptic curve group, incrementing the
        // index, however per "Child key derivation (CKD) functions":
        // https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#child-key-derivation-ckd-functions
        //
        // > "Note: this has probability lower than 1 in 2^127."
        //
        // ...so instead, we simply return an error if this were ever to happen,
        // as the chances of it happening are vanishingly small.
        let key = key.derive_child(child_key.try_into()?)?;

        Ok((key, chain_code.try_into()?))
    }

    pub fn derive_private_key(
        private_key: &SecretKey,
        attrs: &ExtendedKeyAttrs,
        child_number: ChildNumber,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let fingerprint = get_fingerprint(private_key);

        let hmac = Self::create_hmac(private_key, attrs, child_number.is_hardened())?;

        let (private_key, chain_code) = Self::derive_key(private_key, child_number, hmac)?;

        let depth = attrs
            .depth
            .checked_add(1)
            .ok_or(karlsen_bip32::Error::Depth)?;

        let attrs = ExtendedKeyAttrs {
            parent_fingerprint: fingerprint,
            child_number,
            chain_code,
            depth,
        };

        Ok((private_key, attrs))
    }

    fn derive_key(
        private_key: &SecretKey,
        child_number: ChildNumber,
        mut hmac: HmacSha512,
    ) -> Result<(SecretKey, ChainCode)> {
        hmac.update(&child_number.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (child_key, chain_code) = result.split_at(KEY_SIZE);

        // We should technically loop here if a `secret_key` is zero or overflows
        // the order of the underlying elliptic curve group, incrementing the
        // index, however per "Child key derivation (CKD) functions":
        // https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#child-key-derivation-ckd-functions
        //
        // > "Note: this has probability lower than 1 in 2^127."
        //
        // ...so instead, we simply return an error if this were ever to happen,
        // as the chances of it happening are vanishingly small.
        let private_key = private_key.derive_child(child_key.try_into()?)?;

        Ok((private_key, chain_code.try_into()?))
    }

    pub fn create_hmac<K>(
        private_key: &K,
        attrs: &ExtendedKeyAttrs,
        hardened: bool,
    ) -> Result<HmacSha512>
    where
        K: PrivateKey<PublicKey = secp256k1::PublicKey>,
    {
        let mut hmac =
            HmacSha512::new_from_slice(&attrs.chain_code).map_err(karlsen_bip32::Error::Hmac)?;
        if hardened {
            hmac.update(&[0]);
            hmac.update(&private_key.to_bytes());
        } else {
            hmac.update(&private_key.public_key().to_bytes());
        }

        Ok(hmac)
    }

    /// Serialize the raw public key as a byte array.
    pub fn to_bytes(&self) -> PublicKeyBytes {
        self.extended_public_key.to_bytes()
    }

    pub fn attrs(&self) -> &ExtendedKeyAttrs {
        self.extended_public_key.attrs()
    }

    /// Serialize this key as a self-[`Zeroizing`] `String`.
    pub fn to_string(&self, prefix: Option<Prefix>) -> Zeroizing<String> {
        let key = self
            .extended_public_key
            .to_string(Some(prefix.unwrap_or(Prefix::KPUB)));
        Zeroizing::new(key)
    }
}

impl Debug for WalletDerivationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletAccount")
            .field("depth", &self.attrs().depth)
            .field("child_number", &self.attrs().child_number)
            .field(
                "chain_code",
                &faster_hex::hex_string(&self.attrs().chain_code),
            )
            .field("public_key", &faster_hex::hex_string(&self.to_bytes()))
            .field("parent_fingerprint", &self.attrs().parent_fingerprint)
            .finish()
    }
}

#[async_trait]
impl WalletDerivationManagerTrait for WalletDerivationManager {
    /// build wallet from root/master private key
    fn from_master_xprv(
        xprv: &str,
        is_multisig: bool,
        account_index: u64,
        cosigner_index: Option<u32>,
    ) -> Result<Self> {
        let xprv_key = ExtendedPrivateKey::<SecretKey>::from_str(xprv)?;
        let attrs = xprv_key.attrs();

        let (extended_private_key, attrs) = Self::create_extended_key(
            *xprv_key.private_key(),
            attrs.clone(),
            is_multisig,
            account_index,
        )?;

        let extended_public_key = ExtendedPublicKey {
            public_key: extended_private_key.get_public_key(),
            attrs,
        };

        let wallet = Self::from_extended_public_key(extended_public_key, cosigner_index)?;

        Ok(wallet)
    }

    fn from_extended_public_key_str(xpub: &str, cosigner_index: Option<u32>) -> Result<Self> {
        let extended_public_key = ExtendedPublicKey::<secp256k1::PublicKey>::from_str(xpub)?;
        let wallet = Self::from_extended_public_key(extended_public_key, cosigner_index)?;
        Ok(wallet)
    }

    fn from_extended_public_key(
        extended_public_key: ExtendedPublicKey<secp256k1::PublicKey>,
        cosigner_index: Option<u32>,
    ) -> Result<Self> {
        let receive_wallet = Self::derive_child_pubkey_manager(
            extended_public_key.clone(),
            AddressType::Receive,
            cosigner_index,
        )?;

        let change_wallet = Self::derive_child_pubkey_manager(
            extended_public_key.clone(),
            AddressType::Change,
            cosigner_index,
        )?;

        let wallet = Self {
            extended_public_key,
            receive_pubkey_manager: Arc::new(receive_wallet),
            change_pubkey_manager: Arc::new(change_wallet),
        };

        Ok(wallet)
    }

    fn receive_pubkey_manager(&self) -> Arc<dyn PubkeyDerivationManagerTrait> {
        self.receive_pubkey_manager.clone()
    }

    fn change_pubkey_manager(&self) -> Arc<dyn PubkeyDerivationManagerTrait> {
        self.change_pubkey_manager.clone()
    }

    #[inline(always)]
    fn new_receive_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.receive_pubkey_manager.new_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn new_change_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.change_pubkey_manager.new_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn receive_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.receive_pubkey_manager.current_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn change_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.change_pubkey_manager.current_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn derive_receive_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        let key = self.receive_pubkey_manager.derive_pubkey(index)?;
        Ok(key)
    }

    #[inline(always)]
    fn derive_change_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        let key = self.change_pubkey_manager.derive_pubkey(index)?;
        Ok(key)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::{PubkeyDerivationManager, WalletDerivationManager, WalletDerivationManagerTrait};
    use karlsen_addresses::Prefix;

    // these addresses are not karlsen valid addresses (batch replaced)
    fn gen1_receive_addresses() -> Vec<&'static str> {
        vec![
            "karlsen:qzf4vvzvpj9j4frquf4tkd6cwetxx0w9qe2fy8dq26ff2fyhn2wvj46xff3nm",
            "karlsen:qpn9uxfqpjjvm3ejnhz6l28gq3npqwqjfc5efga6r2czz928zae3646rjzf07",
            "karlsen:qrycvrel0y8n5xsrekjtw40qxsfz9qvscxcw7huhv6vuluaje2gzuvq2nzk5m",
            "karlsen:qzuac0fy5k5lg3d55s6q04w49ksaqgngt66cztdujtw77tatkmwngm6gn9l26",
            "karlsen:qqskad0sxl205x2rfgp2ma6h4k0kllwjrvaytrc2wymr39ewfcgpkpzmsv7st",
            "karlsen:qzart5rv9q3h3s4evcvdz2y3xg8gxa92hqez36y4rculdl4vtr5x2hekx2ynr",
            "karlsen:qqu02lxjpsdfwcfmqw8amuw8unqyxurz2g385ujd6fzcud96xh49g9l2wzdh8",
            "karlsen:qrv5mp9afw5h7qdet9mm0v2kxzfcyuxrv5wy6xadljajnfz4zwr37k9xukrvm",
            "karlsen:qrjzdevtzg8mjmk60ymqlcfetyquvkpzs8cx8ler6ylnvdrlnew5ues8cy7x6",
            "karlsen:qzc5h3eheh4wl9juj65t6xsvj4fnecushhjrexnrxe6n8c0pxh996f4gg0lsz",
            "karlsen:qzg63c8fexda24x8a58xsvvnaw772u5hruggvshvutn4wl3lynptzuw6s8ud6",
            "karlsen:qr3jv4fqnscj6hccy0g577mmv6eq3jra8fhve6hvqyzah6rs049wx2dzw0lx2",
            "karlsen:qqvxwx448zz93cm0q5ynesrypfzjy82sp9ju7tsrz7pyk3wn9rlpjzlrytuca",
            "karlsen:qqj0xm5qjn6u42tds24pq4mxqh27847qas25c9u6mfa0gravcj2f2pz2euujn",
            "karlsen:qpp0umu0f6gqaa8j6xd57yj459m2c5g4upaztu5qwqp4qvc2ccpcgk9pu6d66",
            "karlsen:qz3f95hw54nrj8pv4xzxve7leuxvjwtnvu5zj8c579z6f9uzjv6lucsyq6w4v",
            "karlsen:qp8huvhxanvvd8er93g2epff9flhh6aemm82r3mfa8zla0n3xf05k8yf5yd52",
            "karlsen:qrhrlxqnd72jcc6zekflx692qpjyf03zr9ufrp0kr5xqc6gpy273zzjwv83ux",
            "karlsen:qzndcxx23c7w00u3msvhlf763ht7ex56waxjhqn5w7kdp9755y20ulfq60m85",
            "karlsen:qqyfxkl2w724e0a9wz8xx2lwe0v2r2c8y784nn5kvrjd3dz3tx0mylpdm8xv3",
            "karlsen:qz44rhjkrddak9vf5z4swlmenxtfhmqc47d0lyf0j7ednyjln0u824ue33gvr",
        ]
    }

    fn gen1_change_addresses() -> Vec<&'static str> {
        vec![
            "karlsen:qp7yp6h0mm3qculcldu6wm5x4l27p68rvy5kdsc4xd423uyhr7wt6sf4vsjdq",
            "karlsen:qp3gvvff294etymr3dyftvllre4drq7d824a7yhq58hqvkpskx3rcq7m3fswk",
            "karlsen:qzw5t9wxz9le3mjejdhrlmtd0uuhn8m4ml4cm244lc9jhjs8vk3n2hnhk4khs",
            "karlsen:qpht3rv8fdc6unjsduw9jfv4cf0xgd9npcrlg2pkayjrr23j4ljdkz2ny0ld6",
            "karlsen:qz4q6stt5hm5gqh6fgr05j8jf0gm7yrzmhkhw04w7twv7sm539j96vc5lnqur",
            "karlsen:qqjlrpxz5wq4j8amhrxesf9qgtn7kfwnjn33sz9d8e9fpjskqawzcuqdgdh6v",
            "karlsen:qz32h7jwasm85yjjac5xcrsxvpwwyfjs459cx04sur2dqxsw5we5cwfn6aznz",
            "karlsen:qze4wtaz88ve5hm2k0av80jcwrevh8ctuzhz9rnrasdpr0xucn4fsp5lqrd29",
            "karlsen:qzc9lk6whnq3jd09dxfnva9art5d98qqywulam5r2w5lqkvdr4lhkqtxq3xz2",
            "karlsen:qp65xkdn7clpj0vwwnp9jpa22mu549vqe4yyq3wertlzhnsnhl2f7ycuyfvvw",
            "karlsen:qqghlnh2w4pvmrn38fdznnmyc6fyp2udctnl6u3tse8nmz2jk6th559xltvsp",
            "karlsen:qpnac3sms4ev20xtllnm99yxc6xq7z75c6rhwsrkhzexkakft73yjle853n0e",
            "karlsen:qrtg907zyk84vj20z3e2mtxea76npzjak4e9ee7sg87a7p6s20ev7v32k63l3",
            "karlsen:qp94692m3kczqmq4kg9aauh9wlfskckmknjut6hkaezxuvv7dz0v6ue3m79rt",
            "karlsen:qr2pc0vfjm5e6jpexjyvspdf0epj84a9nn906hgsyqgpn0lxnvm92twf28lmv",
            "karlsen:qpfyd49luugxx0hdhclz4sf4r9jelk7r7lcu7nqzu4f353kcuvq6538f50y0m",
            "karlsen:qzeyz98cz43vw5el4dlphdwhqckfxtx0239u42f4qtcv52cgy67hjaced2uv4",
            "karlsen:qzrwzjs45me0ug72g7jc8hf4phjsvstaxa2ea0gja2ejqrg4w25d5xnuaxh3q",
            "karlsen:qz0uujjwyqkj9mmez3732yvtksuerqwv8w0gwp65lenl68ts773cjruuw4tkx",
            "karlsen:qz0wgh8erjjmckxwufwm96u0vmgs34f7axlgasvt2mm9rmxw8c6tkmqthfj0u",
        ]
    }

    #[tokio::test]
    async fn hd_wallet_gen1() {
        let master_xprv =
            "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ";

        let hd_wallet = WalletDerivationManager::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();

        let receive_addresses = gen1_receive_addresses();
        let change_addresses = gen1_change_addresses();

        for index in 0..20 {
            let pubkey = hd_wallet.derive_receive_pubkey(index).unwrap();
            let address: String =
                PubkeyDerivationManager::create_address(&pubkey, Prefix::Mainnet, false)
                    .unwrap()
                    .into();
            assert_eq!(
                receive_addresses[index as usize], address,
                "receive address at {index} failed"
            );
            let pubkey = hd_wallet.derive_change_pubkey(index).unwrap();
            let address: String =
                PubkeyDerivationManager::create_address(&pubkey, Prefix::Mainnet, false)
                    .unwrap()
                    .into();
            assert_eq!(
                change_addresses[index as usize], address,
                "change address at {index} failed"
            );
        }
    }

    #[tokio::test]
    async fn wallet_from_mnemonic() {
        let mnemonic = "fringe ceiling crater inject pilot travel gas nurse bulb bullet horn segment snack harbor dice laugh vital cigar push couple plastic into slender worry";
        let mnemonic =
            karlsen_bip32::Mnemonic::new(mnemonic, karlsen_bip32::Language::English).unwrap();
        let xprv = karlsen_bip32::ExtendedPrivateKey::<karlsen_bip32::SecretKey>::new(
            mnemonic.to_seed(""),
        )
        .unwrap();
        let xprv_str = xprv.to_string(karlsen_bip32::Prefix::KPRV).to_string();
        assert_eq!(
            xprv_str,
            "kprv5y2qurMHCsXYrpeDB395BY2DPKYHUGaCMpFAYRi1cmhwin1bWRyUXVbtTyy54FCGxPnnEvbK9WaiaQgkGS9ngGxmHy1bubZYY6MTokeYP2Q",
            "xprv not matched"
        );

        let wallet = WalletDerivationManager::from_master_xprv(&xprv_str, false, 0, None).unwrap();
        let xpub_str = wallet
            .to_string(Some(karlsen_bip32::Prefix::KPUB))
            .to_string();
        assert_eq!(
            xpub_str,
            "kpub2JM2C9Uh4skDYaLVRddpZBkECgWyRW9kKjgag8MK12X1LACkKR9pAzFzS5hVmWT1oEwxHZxY2AaGg5upMni3HZxoVoAaBQdfPv4UeniLbSx",
            "drived kpub not matched"
        );

        println!("Extended kpub: {}\n", xpub_str);
    }

    #[tokio::test]
    async fn address_test_by_ktrv() {
        let mnemonic = "hunt bitter praise lift buyer topic crane leopard uniform network inquiry over grain pass match crush marine strike doll relax fortune trumpet sunny silk";
        let mnemonic =
            karlsen_bip32::Mnemonic::new(mnemonic, karlsen_bip32::Language::English).unwrap();
        let xprv = karlsen_bip32::ExtendedPrivateKey::<karlsen_bip32::SecretKey>::new(
            mnemonic.to_seed(""),
        )
        .unwrap();
        let ktrv_str = xprv.to_string(karlsen_bip32::Prefix::KTRV).to_string();
        assert_eq!(
            ktrv_str,
            "ktrv5himbbCxArFU2CHiEQyVHP1ABS1tA1SY88CwePzGeM8gHfWmkNBXehhKsESH7UwcxpjpDdMNbwtBfyPoZ7W59kYfVnUXKRgv8UguDns2FQb",
            "master ktrv not matched"
        );

        let wallet = WalletDerivationManager::from_master_xprv(&ktrv_str, false, 0, None).unwrap();
        let ktub_str = wallet
            .to_string(Some(karlsen_bip32::Prefix::KTUB))
            .to_string();
        assert_eq!(
            ktub_str,
            "ktub22zXLBKpG25xeHA8v3ZV9AmhegMZ4SbSXTbMbZiipKSY3uiU914nMjoFPJgfqQ44Hp2XKdsDFKDBDSEhzFmYfnu4nJiot8SQbwY2MfQSEPQ",
            "drived ktub not matched"
        );

        let key = wallet.derive_receive_pubkey(1).unwrap();
        let address = PubkeyDerivationManager::create_address(&key, Prefix::Testnet, false)
            .unwrap()
            .to_string();
        assert_eq!(
            address,
            "karlsentest:qp5xfqdaewr6w0rxs5lrhkzxmjjflkytr9ry0ustmhz0nx46hr3lghvaf7qyt"
        )
    }

    #[tokio::test]
    async fn generate_addresses_by_range() {
        let master_xprv =
            "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ";

        let hd_wallet = WalletDerivationManager::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();
        let pubkeys = hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey_range(0..20)
            .unwrap();
        let addresses_receive = pubkeys
            .into_iter()
            .map(|k| {
                PubkeyDerivationManager::create_address(&k, Prefix::Mainnet, false)
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();

        let pubkeys = hd_wallet
            .change_pubkey_manager()
            .derive_pubkey_range(0..20)
            .unwrap();
        let addresses_change = pubkeys
            .into_iter()
            .map(|k| {
                PubkeyDerivationManager::create_address(&k, Prefix::Mainnet, false)
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();
        println!("receive addresses: {addresses_receive:#?}");
        println!("change addresses: {addresses_change:#?}");
        let receive_addresses = gen1_receive_addresses();
        let change_addresses = gen1_change_addresses();
        for index in 0..20 {
            assert_eq!(
                receive_addresses[index], addresses_receive[index],
                "receive address at {index} failed"
            );
            assert_eq!(
                change_addresses[index], addresses_change[index],
                "change address at {index} failed"
            );
        }
    }

    #[tokio::test]
    async fn generate_karlsentest_addresses() {
        let receive_addresses = [
            "karlsentest:qzf4vvzvpj9j4frquf4tkd6cwetxx0w9qe2fy8dq26ff2fyhn2wvjxqx2eu80",
            "karlsentest:qpn9uxfqpjjvm3ejnhz6l28gq3npqwqjfc5efga6r2czz928zae36xqr3jym2",
            "karlsentest:qrycvrel0y8n5xsrekjtw40qxsfz9qvscxcw7huhv6vuluaje2gzul62sjmq0",
            "karlsentest:qzuac0fy5k5lg3d55s6q04w49ksaqgngt66cztdujtw77tatkmwnggqgs4j7w",
            "karlsentest:qqskad0sxl205x2rfgp2ma6h4k0kllwjrvaytrc2wymr39ewfcgpkjcmnunyl",
            "karlsentest:qzart5rv9q3h3s4evcvdz2y3xg8gxa92hqez36y4rculdl4vtr5x2yrk96f8h",
            "karlsentest:qqu02lxjpsdfwcfmqw8amuw8unqyxurz2g385ujd6fzcud96xh49gk92djqrn",
            "karlsentest:qrv5mp9afw5h7qdet9mm0v2kxzfcyuxrv5wy6xadljajnfz4zwr379lxlxwc0",
            "karlsentest:qrjzdevtzg8mjmk60ymqlcfetyquvkpzs8cx8ler6ylnvdrlnew5u228m5njw",
            "karlsentest:qzc5h3eheh4wl9juj65t6xsvj4fnecushhjrexnrxe6n8c0pxh99660gtljyk",
            "karlsentest:qzg63c8fexda24x8a58xsvvnaw772u5hruggvshvutn4wl3lynptz056nh3ew",
            "karlsentest:qr3jv4fqnscj6hccy0g577mmv6eq3jra8fhve6hvqyzah6rs049wxehzdljj7",
            "karlsentest:qqvxwx448zz93cm0q5ynesrypfzjy82sp9ju7tsrz7pyk3wn9rlpj39r8m3vf",
            "karlsentest:qqj0xm5qjn6u42tds24pq4mxqh27847qas25c9u6mfa0gravcj2f2jc26v3x8",
            "karlsentest:qpp0umu0f6gqaa8j6xd57yj459m2c5g4upaztu5qwqp4qvc2ccpcg9lpl2qww",
            "karlsentest:qz3f95hw54nrj8pv4xzxve7leuxvjwtnvu5zj8c579z6f9uzjv6lut2yr2rpc",
            "karlsentest:qp8huvhxanvvd8er93g2epff9flhh6aemm82r3mfa8zla0n3xf05k57fh5qq7",
            "karlsentest:qrhrlxqnd72jcc6zekflx692qpjyf03zr9ufrp0kr5xqc6gpy273z3gw0hugj",
            "karlsentest:qzndcxx23c7w00u3msvhlf763ht7ex56waxjhqn5w7kdp9755y20uvnqelknq",
            "karlsentest:qqyfxkl2w724e0a9wz8xx2lwe0v2r2c8y784nn5kvrjd3dz3tx0myvmdchtc9",
        ];

        let master_xprv =
            "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ";

        let hd_wallet = WalletDerivationManager::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();

        //let mut receive_addresses = vec![]; //gen1_receive_addresses();
        //let change_addresses = gen1_change_addresses();

        for index in 0..20 {
            let key = hd_wallet.derive_receive_pubkey(index).unwrap();
            //let address = Address::new(Prefix::Testnet, karlsen_addresses::Version::PubKey, key.to_bytes());
            let address =
                PubkeyDerivationManager::create_address(&key, Prefix::Testnet, false).unwrap();
            //receive_addresses.push(String::from(address));
            assert_eq!(
                receive_addresses[index as usize],
                address.to_string(),
                "receive address at {index} failed"
            );
            //let address: String = hd_wallet.derive_change_address(index).await.unwrap().into();
            //assert_eq!(change_addresses[index as usize], address, "change address at {index} failed");
        }

        println!("receive_addresses: {receive_addresses:#?}");
    }
}
