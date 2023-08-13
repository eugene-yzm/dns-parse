use std::error::Error;
use std::vec;
use std::str;
use hex;
use nom::bits::complete::take;
use nom::combinator::map_res;
use nom::IResult;


#[derive(Debug)]
struct Header {
    id: u16, // A 16 bit generated identifier
    qr: bool, // query (0), or a response (1)
    opcode: Opcode, 
    aa: bool, //  Authoritative Answer
    tc: bool, // TrunCation
    rd: bool, // Recursion Desired
    ra: bool, // Recursion Available
    // z Reserved for future use, Must be zero
    rcode: Rcode, // Response code
    qdcount: u16, // specifies the number of entries in the question section
    ancount: u16, // specifies the number of resource records in the answer section 
    nscount: u16, // specifies the number of name server resource records in the authority records section
    arcount: u16 // specifyies the number of resource records in the additional records section
}

#[derive(Debug)]
enum Opcode {
    QUERY, //0               a standard query (QUERY)
    IQUERY, //1               an inverse query (IQUERY)
    STATUS, //2               a server status request (STATUS)
    ERR
    //3-15            reserved for future use
}

impl Opcode {

    fn get_val(v: u8) -> Result<Opcode, anyhow::Error> {
        let oc= match v {
            0 => Opcode::QUERY,
            1 => Opcode::IQUERY,
            2 => Opcode::STATUS,
            _ => Opcode::ERR
        };
        Ok(oc)
    }
}

#[derive(Debug)]
enum Rcode {
    NoErr, // 0               No error condition
    FormatErr, // 1               Format error
    ServerErr, // 2               Server failure
    NameErr, // 3               Name Error
    NoImpl, // 4               Not Implemented
    Refused, // 5               Refused
    //6-15            Reserved for future use
}

fn main() {
    // let s = hex::decode("658c81800001000100000001077477697474657203636f6d0000010001c00c00010001000001d8000468f42a410000291000000000000000").expect("some err");
    // println!("{:?}", s);

    let data = vec![0x65, 0x8c, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x74, 0x77, 0x69, 0x74, 0x74, 0x65, 0x72, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0xd8, 0x00, 0x04, 0x68, 0xf4, 0x2a, 0x41, 0x00, 0x00, 0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let data = hex::decode("658c81800001000100000001077477697474657203636f6d0000010001c00c00010001000001d8000468f42a410000291000000000000000").expect("some err");

    // for ch in hex::decode("60000000009b114026037000a600cd1f").expect("some err") {
    //     print!("{:}", ch as u8);
    // }

    let s = format!("{:?}", &data);
    println!("{}", s);

    let ds = &data[..];
    parse(ds);

    let response: Vec<u8> = vec![];
}

type NomBits<'a> = (&'a [u8], usize);
     
fn parse<'a>(input: &'a[u8]) -> IResult<&'a[u8], Header> {
    nom::bits::bits(parse_header)(input)
}

fn parse_header(input: NomBits) -> IResult<NomBits, Header> {
    // 1  1  1  1  1  1
    // 0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |                      ID                       |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |                    QDCOUNT                    |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |                    ANCOUNT                    |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |                    NSCOUNT                    |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    // |                    ARCOUNT                    |
    // +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

    let (input, id) = nom16(input)?;
    let (input, qr) = nom1(input)?;

    // let (input, _opcode) = nom4(input)?;
    let (input, opcode) = map_res(nom4, Opcode::get_val)(input)?;

    let (input, aa) = nom1(input)?;
    let (input, tc) = nom1(input)?;
    let (input, rd) = nom1(input)?;
    let (mut input, ra) = nom1(input)?;

    for _i in 0..3 {
        let z;
        (input, z) = nom1(input)?;
        assert!(z == false);
    };

    let (input, _rcode) = nom4(input)?;


    println!("remaining input {:?}", input);

    let (input, qdcount) = nom16(input)?;
    let (input, ancount) = nom16(input)?;
    let (input, nscount) = nom16(input)?;
    let (input, arcount) = nom16(input)?;

    let header = Header {
        id,
        qr,
        opcode,
        aa,
        tc,
        rd,
        ra,
        rcode: Rcode::NoErr,
        qdcount,
        ancount,
        nscount,
        arcount
    };
    
    println!("remaining input {:?}", input);
    println!("{:?}", header);
    Ok((input, header))
}

fn nom16(b: NomBits) -> IResult<NomBits, u16> {
    take(16u8)(b)
}

fn nom4(b: NomBits) -> IResult<NomBits, u8> {
    take(4u8)(b)
}

fn nom1(b: NomBits) -> IResult<NomBits, bool> {
    let (b, bit): (NomBits, u8) = take(1u8)(b)?;
    Ok((b, bit == 1))
}