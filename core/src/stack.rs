use crate::Result;

pub struct Stack<const SIZE: usize> {
    sp: usize,
    data: [usize; SIZE],
}

impl<const SIZE: usize> Stack<SIZE> {
    pub fn new() -> Self {
        Self {
            sp: 0,
            data: [0; SIZE],
        }
    }

    pub fn push(&mut self, value: usize) -> Result<()> {
        if self.sp >= SIZE {
            return Err("stack overflow".into());
        }

        self.data[self.sp] = value;
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<usize> {
        if self.sp == 0 {
            return Err("stack underflow".into());
        }

        self.sp -= 1;
        Ok(self.data[self.sp])
    }

    pub fn reset(&mut self) {
        self.sp = 0;
        self.data.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() -> Result<()> {
        let mut stack = Stack::<12>::new();
        stack.push(7)?;
        stack.push(12)?;

        assert_eq!(stack.pop()?, 12);
        assert_eq!(stack.pop()?, 7);
        Ok(())
    }

    #[test]
    fn test_reset() -> Result<()> {
        let mut stack = Stack::<12>::new();
        stack.push(7)?;
        stack.push(10)?;
        stack.reset();

        assert_eq!(stack.sp, 0);
        Ok(())
    }

    #[test]
    fn test_overflow() -> Result<()> {
        let mut stack = Stack::<3>::new();
        stack.push(6)?;
        stack.push(9)?;
        stack.push(21)?;

        assert!(stack.push(7).is_err());
        Ok(())
    }

    #[test]
    fn test_underflow() -> Result<()> {
        let mut stack = Stack::<16>::new();

        assert!(stack.pop().is_err());
        Ok(())
    }
}