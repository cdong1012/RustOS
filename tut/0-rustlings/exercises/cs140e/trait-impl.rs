// FIXME: Make me pass! Diff budget: 25 lines.

<<<<<<< HEAD
#[derive(Debug)]
=======
// I AM NOT DONE

>>>>>>> skeleton/lab2
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}
<<<<<<< HEAD
impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        let mut mili1 = 1u64;
        match *self {
            Duration::MilliSeconds(ms) => mili1 = ms,
            Duration::Seconds(s)       => mili1 = s as u64 * 1000,
            Duration::Minutes(m)       => mili1 = m as u64 * 60 * 1000,
        }
        let mut mili2 = 1u64;
        match *other {
            Duration::MilliSeconds(ms) => mili2 = ms,
            Duration::Seconds(s)       => mili2 = s as u64 * 1000,
            Duration::Minutes(m)       => mili2 = m as u64 * 60 * 1000,
        }
        mili1 == mili2
    }
}
// What traits does `Duration` need to implement?
=======
>>>>>>> skeleton/lab2

#[test]
fn traits() {
    assert_eq!(Seconds(120), Minutes(2));
    assert_eq!(Seconds(420), Minutes(7));
    assert_eq!(MilliSeconds(420000), Minutes(7));
    assert_eq!(MilliSeconds(43000), Seconds(43));
}
