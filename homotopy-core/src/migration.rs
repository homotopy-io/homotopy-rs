use std::{collections::HashMap, io, io::prelude::*};

use base64::decode;
use flate2::bufread::ZlibDecoder;
use serde::Deserialize;
use serde_json::{from_value, Value};
use thiserror::Error;

use crate::{rewrite::Cone, Cospan, Diagram, DiagramN, Generator, Rewrite, Rewrite0, RewriteN};

#[derive(Deserialize)]
pub struct OldProof {
    head: usize,
    #[serde(rename = "entries")]
    _entries: usize,
    #[serde(rename = "index_to_stored_array")]
    stored: Vec<Value>,
    #[serde(skip)]
    pub generator_info: Vec<OldGeneratorInfo>,
    #[serde(skip)]
    pub workspace: Option<OldWorkspace>,
    #[serde(skip)]
    generators: HashMap<usize, Generator>,
    #[serde(skip)]
    diagrams: HashMap<usize, Diagram>,
    #[serde(skip)]
    cones: HashMap<usize, Cone>,
    #[serde(skip)]
    rewrites: HashMap<usize, Rewrite>,
}

pub struct OldGeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub color: String,
    pub diagram: Diagram,
}

pub struct OldWorkspace {
    pub diagram: Diagram,
    // pub path: Vec<SliceIndex>,
    // pub view: u8,
}

impl OldProof {
    pub fn new(proof_str: &str) -> Result<Self> {
        // create an old proof object from string

        // atob()
        let proof_str = decode(proof_str)?;
        // zlib decompress
        let proof_str = decode_bufreader(&proof_str)?;
        // parse json
        let mut proof: OldProof = serde_json::from_str(&proof_str)?;

        proof.load()?;
        Ok(proof)
    }

    fn load(&mut self) -> Result<()> {
        let generators = self.generators()?;
        for v in generators {
            let i: usize = from_value(v["_l"].clone())?;
            self.load_generator(i)?;
        }
        self.load_workspace()?;
        Ok(())
    }

    // returns a list of indices to generators and caches them
    fn generators(&mut self) -> Result<Vec<Value>> {
        let i: usize = from_value(self.stored[self.head][1]["f"]["signature"]["_l"].clone())?;
        let i: usize = from_value(self.stored[i][1]["f"]["generators"]["_l"].clone())?;
        let generators = self.generate_vec(i)?;

        for v in generators.clone() {
            let index: usize = from_value(v["_l"].clone())?;
            let info_index: usize =
                from_value(self.stored[index][1]["f"]["generator"]["_l"].clone())?;
            let id: String = from_value(self.stored[info_index][1]["f"]["id"].clone())?;
            let id: usize = id.parse().unwrap();
            let dim: usize = from_value(self.stored[info_index][1]["n"].clone())?;

            self.generators.insert(id, Generator::new(id, dim));
        }

        Ok(generators)
    }

    // turns an encoded array at [index] to a vec
    fn generate_vec(&mut self, index: usize) -> Result<Vec<Value>> {
        let map: HashMap<String, Value> = from_value(self.stored[index][1]["f"].clone())?;
        let mut v: Vec<(usize, Value)> = map
            .into_iter()
            .map(|x| (x.0.parse().unwrap(), x.1))
            .collect();
        v.sort_by(|x, y| x.0.cmp(&y.0));
        let v: Vec<Value> = v.into_iter().map(|x| x.1).collect();
        Ok(v)
    }

    fn load_generator(&mut self, index: usize) -> Result<()> {
        // Extract generator information
        let name: String = from_value(self.stored[index][1]["f"]["name"].clone())?;
        let color: String = from_value(self.stored[index][1]["f"]["color"].clone())?;

        //log::debug!("loading {}", name);

        let info_index: usize = from_value(self.stored[index][1]["f"]["generator"]["_l"].clone())?;
        let id: String = from_value(self.stored[info_index][1]["f"]["id"].clone())?;
        let id: usize = id.parse().unwrap();
        let diagram_index: usize =
            from_value(self.stored[info_index][1]["f"]["diagram"]["_l"].clone())?;
        let diagram = self.load_diagram(diagram_index)?;

        // Add the diagram into cache
        self.diagrams.insert(diagram_index, diagram.clone());

        // Format generator info
        let info = OldGeneratorInfo {
            generator: self.generators[&id],
            name,
            color,
            diagram,
        };

        // Insert
        self.generator_info.push(info);
        Ok(())
    }

    fn load_diagram(&mut self, index: usize) -> Result<Diagram> {
        if let Some(diagram) = self.diagrams.get(&index) {
            return Ok(diagram.clone());
        }

        let dim: usize = from_value(self.stored[index][1]["n"].clone())?;
        let diagram: Diagram = {
            if dim == 0 {
                let id: String = from_value(self.stored[index][1]["f"]["id"].clone())?;
                let id: usize = id.parse().unwrap();
                self.generators[&id].into()
            } else {
                // Load source
                let source_index: usize =
                    from_value(self.stored[index][1]["f"]["source"]["_l"].clone())?;
                let source = match self.diagrams.get(&source_index) {
                    Some(source) => source.clone(),
                    None => self.load_diagram(source_index)?,
                };

                //log::debug!("source of {} : {:#?}", index, source);

                // Load cospans
                let cospans_index: usize =
                    from_value(self.stored[index][1]["f"]["data"]["_l"].clone())?;
                let cospans_data = self.generate_vec(cospans_index)?;

                let mut cospans: Vec<Cospan> = Vec::new();
                for v in cospans_data {
                    let i: usize = from_value(v["_l"].clone())?;
                    let c = self.load_cospan(i)?;
                    cospans.push(c);
                }

                log::debug!("forming diagram {}", index);
                DiagramN::new(source, cospans).into()
            }
        };

        self.diagrams.insert(index, diagram.clone());
        Ok(diagram)
    }

    fn load_cospan(&mut self, index: usize) -> Result<Cospan> {
        let forward_index = from_value(self.stored[index][1]["f"]["forward_limit"]["_l"].clone())?;
        let backward_index =
            from_value(self.stored[index][1]["f"]["backward_limit"]["_l"].clone())?;

        //log::debug!("loading forward rewrite {}", forward_index);
        let forward = match self.rewrites.get(&forward_index) {
            Some(f) => f.clone(),
            None => self.load_rewrite(forward_index)?,
        };
        //log::debug!("loading forward rewrite {}", backward_index);
        let backward = match self.rewrites.get(&backward_index) {
            Some(b) => b.clone(),
            None => self.load_rewrite(backward_index)?,
        };

        Ok(Cospan { forward, backward })
    }

    fn load_rewrite(&mut self, index: usize) -> Result<Rewrite> {
        if let Some(rewrite) = self.rewrites.get(&index) {
            return Ok(rewrite.clone());
        }

        // Get rewrite information
        let dim: usize = from_value(self.stored[index][1]["n"].clone())?;
        let cones_index: usize =
            from_value(self.stored[index][1]["f"]["components"]["_l"].clone())?;
        let cones_data = self.generate_vec(cones_index)?;

        let rewrite: Rewrite = {
            if dim == 0 {
                let i: usize = from_value(cones_data[0]["_l"].clone())?;

                let source: String = from_value(self.stored[i][1]["f"]["source_id"].clone())?;
                let source: usize = source.parse().unwrap();
                let source = self.generators[&source];

                let target: String = from_value(self.stored[i][1]["f"]["target_id"].clone())?;
                let target: usize = target.parse().unwrap();
                let target = self.generators[&target];

                Rewrite0::new(source, target).into()
            } else {
                let mut cones: Vec<Cone> = Vec::new();
                for v in cones_data {
                    let i: usize = from_value(v["_l"].clone())?;
                    let c = self.load_cone(i)?;
                    cones.push(c);
                }
                RewriteN::new(dim, cones).into()
            }
        };

        self.rewrites.insert(index, rewrite.clone());
        Ok(rewrite)
    }

    fn load_cone(&mut self, index: usize) -> Result<Cone> {
        if let Some(cone) = self.cones.get(&index) {
            return Ok(cone.clone());
        }

        // Get cone information
        let source_index = from_value(self.stored[index][1]["f"]["source_data"]["_l"].clone())?;
        let target_index = from_value(self.stored[index][1]["f"]["target_data"]["_l"].clone())?;
        let slices_index = from_value(self.stored[index][1]["f"]["sublimits"]["_l"].clone())?;
        let cone_index = from_value(self.stored[index][1]["f"]["first"].clone())?;

        // Source and target
        let source_data = self.generate_vec(source_index)?;
        let mut source: Vec<Cospan> = Vec::new();
        for v in source_data {
            let i: usize = from_value(v["_l"].clone())?;
            let c = self.load_cospan(i)?;
            source.push(c);
        }
        let target = self.load_cospan(target_index)?;

        // Slices
        let sublimits = self.generate_vec(slices_index)?;
        let mut slices: Vec<Rewrite> = Vec::new();
        for v in sublimits {
            let i: usize = from_value(v["_l"].clone())?;
            let c = self.load_rewrite(i)?;
            slices.push(c);
        }

        let cone = Cone::new(cone_index, source, target, slices);
        self.cones.insert(index, cone.clone());
        Ok(cone)
    }

    fn load_workspace(&mut self) -> Result<()> {
        // Extract workspace information
        let i: usize = from_value(self.stored[self.head][1]["f"]["workspace"]["_l"].clone())?;
        let diagram_index: usize = match from_value(self.stored[i][1]["f"]["diagram"]["_l"].clone())
        {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };
        //let slices_index = from_value(self.stored[i][1]["f"]["slice"]["_l"].clone())?;
        //let projection = from_value(self.stored[i][1]["f"]["projection"].clone())?;

        // load diagram
        let diagram = self.load_diagram(diagram_index)?;

        // TODO: load path, view

        self.workspace = Some(OldWorkspace { diagram });

        Ok(())
    }
}

type Result<T> = std::result::Result<T, OldProofError>;

#[derive(Debug, Error)]
pub enum OldProofError {
    #[error("JSON parse failed")]
    Parse(serde_json::Error),
    #[error("cannot decode base64")]
    Decode(base64::DecodeError),
    #[error("cannot decompress proof string")]
    Io(std::io::Error),
    #[error("internal error")]
    Internal,
}

impl From<serde_json::Error> for OldProofError {
    fn from(err: serde_json::Error) -> OldProofError {
        OldProofError::Parse(err)
    }
}

impl From<base64::DecodeError> for OldProofError {
    fn from(err: base64::DecodeError) -> OldProofError {
        OldProofError::Decode(err)
    }
}

impl From<std::io::Error> for OldProofError {
    fn from(err: std::io::Error) -> OldProofError {
        OldProofError::Io(err)
    }
}

fn decode_bufreader(bytes: &[u8]) -> io::Result<String> {
    let mut z = ZlibDecoder::new(bytes);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}
