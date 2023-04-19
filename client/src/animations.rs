/// Copied from my game Vupa
#[derive(PartialEq, Eq)]
pub enum DelayState<T> {
    // Timeline: --------------------------------
    //       delay ended here|    |but `ended()` is called here
    // We return the time between the two bars
    Done(T), //Time since done
    Running,
}
pub struct StringAnimation {
    interval: SystemTimeDelay,
    txt: String,
    // head: usize,
    // dir: bool,
    one: usize,
    two: usize,
}

/// Copied from my game Vupa
#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemTimeDelay {
    instant: std::time::Instant,
    timeout: u128,
}

impl StringAnimation {
    pub fn new(interval_delay: u128, txt: String) -> Self {
        Self {
            interval: SystemTimeDelay::from(interval_delay),
            txt,
            // head: 0,
            // dir: false,
            one: 0,
            two: 0,
        }
    }
    pub fn get(&mut self) -> String {
        // -v this was a good idea, but if fixed the original idea, so i'll go with it for now

        // if let DelayState::Done(_) = self.interval.ended() {
        //     if self.dir {
        //         if self.head > self.txt.len() - 1 {
        //             // self.head = 0;
        //             self.dir = !self.dir;
        //         } else {
        //             self.head += 1;
        //         }
        //     } else {
        //         if self.head == 0 {
        //             // self.head = self.txt.len();
        //             self.dir = !self.dir;
        //         } else {
        //             self.head -= 1;
        //         }
        //     }
        //     debug!("{}", self.head);
        //     self.interval.restart()
        // }
        // let txt_size = self.txt.len();

        // let v_total = unsafe { self.txt.as_mut_vec() };
        // let substring = &v_total[..self.head];

        // let sub =
        //     std::str::from_utf8(substring).unwrap().to_owned() + &"  ".repeat(txt_size - self.head);

        let txt_size = self.txt.len();

        let v_total = unsafe { self.txt.as_mut_vec() };
        let sliced = std::str::from_utf8(&v_total[self.two..self.one]).unwrap();

        let sub = /* " ".repeat(txt_size - self.two) + */ sliced.to_owned()+ &" ".repeat(txt_size - self.one);

        if let DelayState::Done(_) = self.interval.ended() {
            if self.one > txt_size - 1 {
                self.two += 1;
                if self.two > txt_size - 1 {
                    self.one = 0;
                    self.two = 0
                }
            } else {
                self.one += 1
            }
            self.interval.restart()
        }
        sub
    }
}

/// Copied from my game Vupa (and trimmed a ton to keep only the usefull part)
impl SystemTimeDelay {
    pub fn new(timeout: u128) -> Self {
        Self {
            instant: std::time::Instant::now(),
            timeout,
        }
    }

    pub fn restart(&mut self) {
        *self = Self::new(self.timeout)
    }
    pub fn ended(&self) -> DelayState<u128> {
        let e = self.instant.elapsed().as_millis();

        if e >= self.timeout {
            // elapsed?, how much ms has passed since elapsed
            DelayState::Done(e - self.timeout)
        } else {
            DelayState::Running
        }
    }
}

impl From<u128> for SystemTimeDelay {
    fn from(timeout: u128) -> SystemTimeDelay {
        SystemTimeDelay::new(timeout)
    }
}
