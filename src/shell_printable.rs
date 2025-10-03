use std::str::Utf8Error;

pub trait ShellPrintable {
    fn printable_invocation_string(&self) -> Result<String, Utf8Error>;
    // Calls `.to_string_lossy()` on the program name and args.
    fn printable_invocation_string_lossy(&self) -> String;
}

pub trait ShellPrintableRef: ShellPrintable {
    // Print the invocation to `stdout`.`
    fn print_invocation(&mut self) -> Result<&mut Self, Utf8Error> {
        println!("{}", self.printable_invocation_string_lossy());
        Ok(self)
    }
    // Print the invocation to `stdout`.`
    fn print_invocation_lossy(&mut self) -> &mut Self {
        println!("{}", self.printable_invocation_string_lossy());
        self
    }
}

pub trait ShellPrintableSelf: ShellPrintable {
    // Print the invocation to `stdout`.`
    fn print_invocation(self) -> Result<Self, Utf8Error>
    where
        Self: std::marker::Sized,
    {
        println!("{}", self.printable_invocation_string_lossy());
        Ok(self)
    }
    // Print the invocation to `stdout`.`
    fn print_invocation_lossy(self) -> Self
    where
        Self: std::marker::Sized,
    {
        println!("{}", self.printable_invocation_string_lossy());
        self
    }
}
