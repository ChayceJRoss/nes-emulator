//! # CPU Module
//!
//! `cpu` implements the hardware and ALU for the cpu in this project.


/// This struct implements the hardware available to the NES in the CPU.
pub struct CPU {
    pub register_a : u8,
    pub register_x : u8,
    pub register_y : u8,
    pub status : u8,
    pub program_counter : u16,
    memory : [u8 ; 0xFFFF]
}


/// This enum allows matching against the different available addressing modes for each opcode. [This](https://skilldrick.github.io/easy6502/#addressing) resource more
/// information about addressing modes 
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
    /// Initialises the CPU, all registers and memory addresses are initialised with 0x00.
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

    /// Matches the addressing mode provided by the opcode, returns the absolute address of the memory to
    /// be accessed. 
    /// 
    /// Note that this is a poor analogy for the an actual CPU as the there are no cycle, or space saves 
    /// for using paged references. 
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

    /// Reads the the byte from the memory address. 
    fn mem_read(&self, address : u16) -> u8 {
        self.memory[address as usize]
    }

    /// Reads two bytes from the provided address and the next address, note that the bytes returned use little endian 
    /// notation (i.e. pos -> LSB, pos + 1 -> MSB). 
    fn mem_read_u16(&self, pos : u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    /// Writes a byte to memory at provided absolute address. 
    fn mem_write(&mut self, address : u16, data : u8) {
        self.memory[address as usize] = data;
    }


    /// Writes two bytes starting at position provided using little endian addressing. (i.e. pos = LSB, pos + 1 = MSB). 
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }


    /// Loads (see [`crate::cpu::CPU::load`]), to CPU, resets (see [`crate::cpu::CPU::reset`]) the CPU, and runs (see [`crate::cpu::CPU::run`]) the program.
    /// 
    /// # Example 
    /// This program loads the A register with 0x01 and then moves it to X register, finally ending the program.
    /// ```
    ///  use nes::cpu::CPU;  
    ///  
    ///  let mut cpu = CPU::new();
    ///  cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
    ///  assert_eq!(cpu.register_x, 1);
    /// ```
    pub fn load_and_run(&mut self, program : Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    /// Sets all registers to 0x00 and then moves the program counter to the absolute address referenced by the bytes stored at 0xFFFC and 0xFFFD. 
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
    
        self.program_counter = self.mem_read_u16(0xFFFC);
    }


    /// Loads a program (vector of opcodes) to 0x8000 to 0x8000 + length of program. Sets the program start bytes at 0xFFFC and 0xFFFD to 0x8000.
    pub fn load(&mut self, program : Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    /// Loads a byte into A register
    fn lda(&mut self, value : u8) {
        self.register_a = value;
        self.update_zero_and_negative(value)
    }

    /// Loads the byte stored in A register to X register
    fn tax (&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative(self.register_x);
    }

    /// Increments (with wrapping) the byte stored in the X register.
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative(self.register_x);
    }

    /// This is used to update the status register zero and negative flags.
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

    /// Runs a program by iteratively incrementing the program counter until the exit code is reached (0x00)
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