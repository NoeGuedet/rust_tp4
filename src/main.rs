use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use chrono::{DateTime, Local};

// Structure pour gérer les fichiers
struct FileManager {
    current_file: Option<String>,
    last_modified: Option<DateTime<Local>>,
}

impl FileManager {
    fn new() -> Self {
        FileManager {
            current_file: None,
            last_modified: None,
        }
    }

    fn read_file(&mut self, filename: &str) -> io::Result<String> {
        self.current_file = Some(filename.to_string());
        
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        println!("Fichier lu avec succès: {}", filename);
        self.last_modified = Some(Local::now());
        
        Ok(content)
    }

    fn write_file(&mut self, filename: &str, content: &str) -> io::Result<()> {
        self.current_file = Some(filename.to_string());
        
        let mut file = File::create(filename)?;
        file.write_all(content.as_bytes())?;
        
        println!("Fichier écrit avec succès: {}", filename);
        self.last_modified = Some(Local::now());
        
        Ok(())
    }

    fn modify_file(&mut self, filename: &str, content: &str) -> io::Result<()> {
        self.current_file = Some(filename.to_string());
        
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filename)?;
            
        file.write_all(content.as_bytes())?;
        
        println!("Fichier modifié avec succès: {}", filename);
        self.last_modified = Some(Local::now());
        
        Ok(())
    }

    fn delete_file(&mut self, filename: &str) -> io::Result<()> {
        if Path::new(filename).exists() {
            fs::remove_file(filename)?;
            println!("Fichier supprimé avec succès: {}", filename);
            
            if let Some(current) = &self.current_file {
                if current == filename {
                    self.current_file = None;
                }
            }
            
            self.last_modified = Some(Local::now());
            Ok(())
        } else {
            println!("Le fichier n'existe pas: {}", filename);
            Err(io::Error::new(io::ErrorKind::NotFound, "Fichier non trouvé"))
        }
    }

    // Afficher les informations sur le fichier actuel
    fn display_info(&self) {
        match &self.current_file {
            Some(filename) => {
                println!("Fichier actuel: {}", filename);
                if let Some(date) = self.last_modified {
                    println!("Dernière modification: {}", date.format("%Y-%m-%d %H:%M:%S"));
                }
            },
            None => println!("Aucun fichier n'est actuellement sélectionné.")
        }
    }
}

fn main() {
    let mut file_manager = FileManager::new();
    let mut running = true;

    println!("=== Gestionnaire de Fichiers ===");
    
    while running {
        println!("\nChoisissez une option:");
        println!("1. Lire un fichier");
        println!("2. Écrire dans un fichier");
        println!("3. Modifier un fichier");
        println!("4. Supprimer un fichier");
        println!("5. Afficher les informations");
        println!("6. Quitter");
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect(
            "Échec de la lecture de l'entrée, ca doit être une option entre 1 et 6"
        );
        
        match choice.trim() {
            "1" => {
                println!("Entrez le nom du fichier à lire:");
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).expect("Échec de la lecture de l'entrée");
                
                match file_manager.read_file(filename.trim()) {
                    Ok(content) => println!("Contenu du fichier:\n{}", content),
                    Err(e) => println!("Erreur lors de la lecture du fichier: {}", e),
                }
            },
            "2" => {
                println!("Entrez le nom du fichier à écrire:");
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).expect("Échec de la lecture de l'entrée");
                
                println!("Entrez le contenu à écrire:");
                let mut content = String::new();
                io::stdin().read_line(&mut content).expect("Échec de la lecture de l'entrée");
                
                match file_manager.write_file(filename.trim(), &content) {
                    Ok(_) => (),
                    Err(e) => println!("Erreur lors de l'écriture du fichier: {}", e),
                }
            },
            "3" => {
                println!("Entrez le nom du fichier à modifier:");
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).expect("Échec de la lecture de l'entrée");
                
                println!("Entrez le contenu à ajouter:");
                let mut content = String::new();
                io::stdin().read_line(&mut content).expect("Échec de la lecture de l'entrée");
                
                match file_manager.modify_file(filename.trim(), &content) {
                    Ok(_) => (),
                    Err(e) => println!("Erreur lors de la modification du fichier: {}", e),
                }
            },
            "4" => {
                println!("Entrez le nom du fichier à supprimer:");
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).expect("Échec de la lecture de l'entrée");
                
                match file_manager.delete_file(filename.trim()) {
                    Ok(_) => (),
                    Err(e) => println!("Erreur lors de la suppression du fichier: {}", e),
                }
            },
            "5" => {
                file_manager.display_info();
            },
            "6" => {
                println!("Au revoir!");
                running = false;
            },
            _ => println!("Option invalide, veuillez réessayer."),
        }
    }
}