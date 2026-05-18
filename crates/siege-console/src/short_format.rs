pub trait ShortFormat {
    fn short(&self) -> String;
}

impl ShortFormat for u32 {
    fn short(&self) -> String {
        let n = *self;
        if n >= 1_000_000 {
            let whole = n / 1_000_000;
            let frac = (n % 1_000_000) / 100_000;
            if frac == 0 {
                format!("{whole}M")
            } else {
                format!("{whole}.{frac}M")
            }
        } else if n >= 1_000 {
            let whole = n / 1_000;
            let frac = (n % 1_000) / 100;
            if frac == 0 {
                format!("{whole}k")
            } else {
                format!("{whole}.{frac}k")
            }
        } else {
            format!("{n}")
        }
    }
}
