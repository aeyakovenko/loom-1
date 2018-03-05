use result::{Error, Result};
use core::marker::PhantomData;

pub trait Key: Eq {
    fn start(&self) -> usize;
    fn unused(&self) -> bool;
}

pub trait Val<T: Key>: Sized + Clone {
    fn key(&self) -> &T;
}

pub struct HashT<K: Key, V: Val<K>> {
    _key: PhantomData<K>,
    _val: PhantomData<V>,
}

impl<K, V> HashT<K, V>
where
    K: Key,
    V: Val<K>,
{
    pub fn find(tbl: &[V], key: &K) -> Result<usize> {
        let num_elems = tbl.len();
        let st = key.start();
        for i in 0..num_elems {
            let pos = st.wrapping_add(i) % num_elems;
            let k = tbl.get(pos).unwrap().key();
            if k.unused() || k == key {
                return Ok(pos);
            }
        }
        Err(Error::NoSpace)
    }
    pub fn migrate(src: &[V], dst: &mut [V]) -> Result<()> {
        for i in src {
            if i.key().unused() {
                continue;
            }
            let p = Self::find(dst, i.key())?;
            *dst.get_mut(p).unwrap() = (*i).clone();
            assert_eq!(Self::find(dst, i.key()).unwrap(), p);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use hasht;
    use result::Error;
    impl hasht::Key for usize {
        fn start(&self) -> usize {
            *self
        }
        fn unused(&self) -> bool {
            *self == 0usize
        }
    }

    impl hasht::Val<usize> for usize {
        fn key(&self) -> &usize {
            &self
        }
    }
    type UsizeT = hasht::HashT<usize, usize>;
    #[test]
    fn hash_test() {
        let v = vec![0usize; 0];
        let r = UsizeT::find(&v, &1usize);
        assert_matches!(r.unwrap_err(), Error::NoSpace);
    }
    #[test]
    fn hash_test2() {
        let v = vec![0usize; 1];
        let r = UsizeT::find(&v, &1usize);
        assert_eq!(r.unwrap(), 0);
    }
    #[test]
    fn hash_test3() {
        let mut v = vec![0usize; 1];
        let a = UsizeT::find(&v, &1usize).expect("find 1");
        v[a] = 1;
        let b = UsizeT::find(&v, &2usize);
        assert_matches!(b.unwrap_err(), Error::NoSpace);
    }
    #[test]
    fn hash_test4() {
        let mut v = vec![0usize; 2];
        let a = UsizeT::find(&v, &1usize).expect("find 1");
        v[a] = 1;
        let b = UsizeT::find(&v, &2usize).expect("find 2");
        assert_ne!(a, b);
        v[b] = 2;
        assert_eq!(UsizeT::find(&v, &1usize).unwrap(), a);
        assert_eq!(UsizeT::find(&v, &2usize).unwrap(), b);
        assert_matches!(UsizeT::find(&v, &3usize).unwrap_err(), Error::NoSpace);
        assert_matches!(
            UsizeT::find(&v, &usize::max_value()).unwrap_err(),
            Error::NoSpace
        );
    }
    #[test]
    fn hash_migrate_test() {
        let mut v = vec![0usize; 2];
        let a = UsizeT::find(&v, &1usize).expect("find 1");
        v[a] = 1;
        let b = UsizeT::find(&v, &2usize).expect("find 2");
        assert_ne!(a, b);
        v[b] = 2;
        assert_eq!(UsizeT::find(&v, &1usize).unwrap(), a);
        assert_eq!(UsizeT::find(&v, &2usize).unwrap(), b);
        let mut m = vec![0usize; 4];
        UsizeT::migrate(&v, &mut m).expect("migrate");
        let ma = UsizeT::find(&m, &1usize).expect("find 1");
        let mb = UsizeT::find(&m, &2usize).expect("find 2");
        assert_eq!(m[ma], 1);
        assert_eq!(m[mb], 2);
        let mc = UsizeT::find(&m, &3usize).expect("find 3");
        m[mc] = 3;
        assert_eq!(UsizeT::find(&m, &3usize).unwrap(), mc);
    }

}
