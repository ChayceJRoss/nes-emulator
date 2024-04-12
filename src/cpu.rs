pub struct CPU {
    pub register_a : u8,
    pub register_x : u8,
    pub register_y : u8,
    pub status : u8,
    pub program_counter : u16,
    memory : [u8 ; 0xFFFF]
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}



impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x : 0,
            register_y : 0,
            status: 0,
            program_counter: 0,
            memory : [0 ; 0xFFFF]
        }
    }

    fn get_operand_address(&self, mode : &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter) as u16,

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            },

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            },

            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_x as u16) as u16;
                addr
            },

            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_y as u16) as u16;
                addr
            },

            AddressingMode::Indirect_X => {
                let zero_page = self.mem_read(self.program_counter);
                let address = zero_page.wrapping_add(self.register_x) as u16;

                let lo = self.mem_read(address) as u16;
                let hi = self.mem_read(address.wrapping_add(1)) as u16;
                (hi << 8) | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
    
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn mem_read(&self, address : u16) -> u8 {
        self.memory[address as usize]
    }

    fn mem_read_u16(&self, pos : u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, address : u16, data : u8) {
        self.memory[address as usize] = data;
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn load_and_run(&mut self, program : Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
    
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program : Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    fn lda(&mut self, value : u8) {
        self.register_a = value;
        self.update_zero_and_negative(value)
    }

    fn tax (&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative(self.register_x);
    }

    fn update_zero_and_negative(&mut self, result : u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                0xA9 => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.lda(param);
                }

                0xAA => self.tax(),

                0xE8 => self.inx(),

                0x00 => {
                    return;
                }
                _ => todo!("")
            }
        }
    }
}