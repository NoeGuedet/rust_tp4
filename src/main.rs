use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::net::UdpSocket;
use std::str;

// ------------ Structures DNS Binaires ------------

#[derive(Debug)]
struct DnsHeader {
    id: u16,
    flags: u16,
    qdcount: u16, // number of questions
    ancount: u16, // number of answers
    nscount: u16,
    arcount: u16,
}

#[derive(Debug)]
struct DnsQuestion {
    qname: String,
    qtype: u16,
    qclass: u16,
}

// Simple answer : only A record
#[derive(Debug)]
struct DnsAnswer {
    name: String,
    atype: u16,
    aclass: u16,
    ttl: u32,
    rdata: [u8; 4], // IPv4 seulement (A)
}

// ------------ DNS (Dé-)Sérialisation ------------

// Écriture d'un nom de domaine (format DNS, label)
fn write_qname<W: Write>(mut w: W, domain: &str) -> std::io::Result<()> {
    for part in domain.split('.') {
        w.write_u8(part.len() as u8)?;
        w.write_all(part.as_bytes())?;
    }
    w.write_u8(0)?; // fin du nom
    Ok(())
}

// Lecture d'un nom de domaine (format DNS, label)
fn read_qname<R: Read>(mut r: R) -> std::io::Result<String> {
    let mut name = String::new();
    loop {
        let len = r.read_u8()?;
        if len == 0 {
            break;
        }
        let mut buf = vec![0; len as usize];
        r.read_exact(&mut buf)?;
        if !name.is_empty() {
            name.push('.');
        }
        name.push_str(str::from_utf8(&buf).unwrap());
    }
    Ok(name)
}

// ------------ Client DNS ------------

fn dns_client(server: &str, domain: &str) -> std::io::Result<()> {
    // Construction header
    let tx_id = 0x1234;
    let mut req = vec![];
    // Header
    req.write_u16::<BigEndian>(tx_id)?; // ID
    req.write_u16::<BigEndian>(0x0100)?; // flags = query + recursion desired
    req.write_u16::<BigEndian>(1)?; // one question
    req.write_u16::<BigEndian>(0)?; // no answer
    req.write_u16::<BigEndian>(0)?; // no authority
    req.write_u16::<BigEndian>(0)?; // no additional
    // Question section
    write_qname(&mut req, domain)?;
    req.write_u16::<BigEndian>(1)?; // type = A
    req.write_u16::<BigEndian>(1)?; // class = IN

    // Envoi de la requête
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&req, server)?;

    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf)?;
    let resp = &buf[..len];

    // Décodage simplifié
    let mut c = Cursor::new(resp);
    let resp_id = c.read_u16::<BigEndian>()?;
    let _flags = c.read_u16::<BigEndian>()?;
    let qdcount = c.read_u16::<BigEndian>()?;
    let ancount = c.read_u16::<BigEndian>()?;
    let _nscount = c.read_u16::<BigEndian>()?;
    let _arcount = c.read_u16::<BigEndian>()?;

    if resp_id != tx_id {
        println!("ID mismatch!");
        return Ok(());
    }

    // Skip Question
    for _ in 0..qdcount {
        let _ = read_qname(&mut c)?;
        let _ = c.read_u16::<BigEndian>()?;
        let _ = c.read_u16::<BigEndian>()?;
    }

    for _ in 0..ancount {
        let _ = read_qname(&mut c)?;
        let atype = c.read_u16::<BigEndian>()?;
        let _aclass = c.read_u16::<BigEndian>()?;
        let _ttl = c.read_u32::<BigEndian>()?;
        let rdlen = c.read_u16::<BigEndian>()?;
        if atype == 1 && rdlen == 4 {
            // A record
            let mut ip = [0u8; 4];
            c.read_exact(&mut ip)?;
            println!("Réponse : {:?} -> {}.{}.{}.{}", domain, ip[0], ip[1], ip[2], ip[3]);
        } else {
            c.seek(SeekFrom::Current(rdlen as i64)).unwrap();
        }
    }
    Ok(())
}

// ------------ Serveur DNS ------------

fn dns_server(listen: &str) -> std::io::Result<()> {
    let socket = UdpSocket::bind(listen)?;
    println!("Serveur DNS écoute sur {}", listen);

    // Domaines -> IPs
    let mut table = HashMap::new();
    table.insert("local.test", [127, 0, 0, 1]);
    table.insert("rust.test", [1, 2, 3, 4]);
    table.insert("exemple.com", [8, 8, 8, 8]);

    loop {
        let mut buf = [0u8; 512];
        let (len, src) = socket.recv_from(&mut buf)?;
        let req = &buf[..len];
        // Analyse rapide du paquet
        let mut c = Cursor::new(req);

        let tx_id = c.read_u16::<BigEndian>()?;
        let _flags = c.read_u16::<BigEndian>()?;
        let qdcount = c.read_u16::<BigEndian>()?;
        let _ancount = c.read_u16::<BigEndian>()?;
        let _nscount = c.read_u16::<BigEndian>()?;
        let _arcount = c.read_u16::<BigEndian>()?;

        let mut resp = Vec::new();
        // Reprend header du client (id, etc.)
        resp.write_u16::<BigEndian>(tx_id)?;
        resp.write_u16::<BigEndian>(0x8180)?; // response + recursion available + no error
        resp.write_u16::<BigEndian>(qdcount)?;
        resp.write_u16::<BigEndian>(if qdcount > 0 { 1 } else { 0 })?; // 1 réponse pour 1 question
        resp.write_u16::<BigEndian>(0)?; // nscount
        resp.write_u16::<BigEndian>(0)?; // arcount

        let mut questions = vec![];
        for _ in 0..qdcount {
            let qname = read_qname(&mut c)?;
            let qtype = c.read_u16::<BigEndian>()?;
            let qclass = c.read_u16::<BigEndian>()?;
            // Reconstruction section question
            write_qname(&mut resp, &qname)?;
            resp.write_u16::<BigEndian>(qtype)?;
            resp.write_u16::<BigEndian>(qclass)?;
            questions.push((qname, qtype, qclass));
        }

        // Section Answer si possible
        for (qname, qtype, _) in &questions {
            if *qtype == 1 {
                if let Some(ip) = table.get(qname.as_str()) {
                    // Answer
                    write_qname(&mut resp, qname)?;
                    resp.write_u16::<BigEndian>(1)?; // type A
                    resp.write_u16::<BigEndian>(1)?; // class IN
                    resp.write_u32::<BigEndian>(60)?; // TTL s
                    resp.write_u16::<BigEndian>(4)?; // data len
                    resp.write_all(ip)?; // IPv4
                }
            }
        }

        socket.send_to(&resp, src)?;
    }
}

// ------------ Main Entrypoint (mode client ou serveur par CLI) ------------

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Utilisation :");
        println!("  Serveur : {} server <ip:port>", args[0]);
        println!("  Client : {} client <ip:port_server> <nom_de_domaine>", args[0]);
        return Ok(());
    }
    match args[1].as_str() {
        "server" if args.len() == 3 => dns_server(&args[2]),
        "client" if args.len() == 4 => dns_client(&args[2], &args[3]),
        _ => {
            println!("Argument invalide.");
            Ok(())
        }
    }
}
