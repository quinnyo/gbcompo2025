use std::collections::{HashMap, HashSet};

use crate::{
    coord::{DVec2, IVec2, U16Vec2, U8Vec2},
    extract::{Extract, ExtractNode, ExtractNodeAccess, ExtractNodeId},
    out::{Chunk, Element},
};

pub mod these_converters {
    use crate::{
        brush::{Brush, Brushes},
        convert::{Conversion, ConvertNodeResult, Importance, ProcessResult, SelectNode},
        coord::IVec2,
        elem::ElemId,
        extract::ExtractNodeId,
        flow::FlowSeqCom,
        geometry::Shape,
        out::{Chunk, Element, ElementType, FlowRules},
        tiled_ext,
    };

    /// Submit converters/processors to the conversion context
    pub fn submit(conversion: &mut Conversion) {
        // Tilemap/chunks (preprocessor)
        conversion.add_preprocessor(Importance::Critical, |conversion, partial| {
            // sort chunk coords row major
            let mut chunk_coords: Vec<IVec2> = conversion.extract.chunks.keys().cloned().collect();
            chunk_coords.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

            // find minimum extent of chunk coords
            let chunk_origin: IVec2 = chunk_coords
                .iter()
                .cloned()
                .reduce(|a, b| a.min(b))
                .unwrap_or_default();
            partial.chunk_origin = Some(chunk_origin);

            // convert chunks
            partial.chunks = chunk_coords
                .iter()
                .map(|old_coord| {
                    let mut chr_brushes: Brushes<u8> = Default::default();
                    let mut atrb_brushes: Brushes<u8> = Default::default();
                    for (_, _, cell) in conversion
                        .extract
                        .chunks
                        .get(old_coord)
                        .unwrap()
                        .tilemap
                        .iter()
                    {
                        let (chr, atrb) = cell
                            .clone()
                            .map(|tile| conversion.extract.tiles.get_chr_atrb(tile))
                            .unwrap_or((0, 0));
                        chr_brushes.push_literal(chr);
                        atrb_brushes.push_literal(atrb);
                    }
                    chr_brushes.push(Brush::Terminator);
                    atrb_brushes.push(Brush::Terminator);
                    let coord = (old_coord - chunk_origin).as_u8vec2();
                    (coord, Chunk::with_tilemap(coord, chr_brushes, atrb_brushes))
                })
                .collect();

            ProcessResult::Success
        });

        // PlayerStart
        conversion.add_node_converter(
            SelectNode::UserType("PlayerStart"),
            |input, _conversion, partial| {
                let world_dots =
                    partial.extract_global_position_to_world_dots(input.global_position());
                partial.add_map_element(Element::new(
                    ElemId::ExtractId(input.id),
                    ElementType::PlayerStart(world_dots),
                ));
                ConvertNodeResult::Consume
            },
        );

        // Marker
        conversion.add_node_converter(
            SelectNode::UserType("Marker"),
            |input, _conversion, partial| {
                let id: i32 = tiled_ext::properties_get(&input.properties, "id").unwrap();
                let world_dots =
                    partial.extract_global_position_to_world_dots(input.global_position());

                partial.add_map_element(Element::new(
                    ElemId::ExtractId(input.id),
                    ElementType::Marker {
                        tag: id as u16,
                        position: world_dots,
                    },
                ));
                ConvertNodeResult::Consume
            },
        );

        // ZFlow (layer)
        conversion.add_node_converter(
            SelectNode::UserType("ZFlow"),
            |layer, conversion, partial| {
                let vecs_magnitude: Vec<f64> =
                    tiled_ext::properties_get_list(&layer.properties, "vecs_magnitude").unwrap();
                let sequence: Vec<FlowSeqCom> =
                    tiled_ext::properties_get_list(&layer.properties, "sequence").unwrap();
                let from_id: ExtractNodeId =
                    tiled_ext::properties_get(&layer.properties, "from").unwrap();
                let from = conversion
                    .extract
                    .access(&from_id)
                    .unwrap()
                    .global_position();
                let to_id: ExtractNodeId =
                    tiled_ext::properties_get(&layer.properties, "to").unwrap();
                let to = conversion.extract.access(&to_id).unwrap().global_position();
                let diff = to - from;
                let dir = diff
                    .try_normalize()
                    .expect("failed to normalise flow vector");
                let vecs = vecs_magnitude
                    .iter()
                    .map(|m| (m * dir).round().as_i8vec2())
                    .collect();
                let rules_id = ElemId::ExtractId(layer.id);
                partial.add_map_element(Element::new(
                    rules_id,
                    ElementType::FlowRules(FlowRules::new(vecs, sequence.clone())),
                ));

                // collect ZArea objects on the ZFlow layer
                for object in layer
                    .objects()
                    .filter(|o| SelectNode::UserType("ZArea").selects(o))
                {
                    // TODO: handle zones that straddle chunks ~~ split into multiple rects, I guess.
                    let size = object
                        .shape
                        .as_ref()
                        .and_then(|s| {
                            if let Shape::Rect { size } = s {
                                Some(size.floor().as_u8vec2())
                            } else {
                                None
                            }
                        })
                        .expect("ZArea must be Rectangle");
                    assert!(size.x > 0 && size.y > 0, "ZArea must have non-zero area");
                    let position =
                        partial.extract_global_position_to_world_dots(object.global_position());
                    let chunk_coord = position / 8 / 16; // chunk location in chunk coordinates
                    let chunk_position = 16 * 8 * chunk_coord; // chunk origin in world
                    partial.add_chunk_element(
                        chunk_coord.as_u8vec2(),
                        Element::new(
                            ElemId::ExtractId(object.id),
                            ElementType::Zone {
                                position: (position - chunk_position).as_u8vec2(),
                                size,
                                rules: rules_id,
                            },
                        ),
                    );
                }

                ConvertNodeResult::ConsumeBranch
            },
        );
    }
}

#[derive(Debug, Default)]
pub struct Partial {
    /// Top-level map resources/elements.
    map_elements: Vec<Element>,
    /// Converted, ordered chunks with corrected coordinates.
    chunks: HashMap<U8Vec2, Chunk>,
    /// The tilemap origin from the source (i.e. before correction) in chunk coords.
    chunk_origin: Option<IVec2>,
}

impl Partial {
    pub fn extract_global_position_to_world_dots(&self, xgpos: DVec2) -> U16Vec2 {
        (xgpos.round().as_ivec2()
            - self
                .chunk_origin
                .expect("Conversion::chunk_origin must be configured")
                * 16
                * 8)
        .as_u16vec2()
    }

    pub fn add_map_element(&mut self, elem: Element) {
        self.map_elements.push(elem)
    }

    pub fn add_chunk_element(&mut self, coord: U8Vec2, elem: Element) {
        self.chunks
            .entry(coord)
            .or_insert_with(|| Chunk::new(coord))
            .push_element(elem)
    }
}

/// Conversion output type
#[derive(Debug, Default)]
pub struct Converted {
    pub resources: Vec<Element>,
    pub chunks: Vec<Chunk>,
}

/// Conversion configuration context
#[derive(Debug, Default)]
pub struct Conversion {
    /// The input extraction context.
    extract: Extract,
    /// First stage conversion processors
    preprocessors: Vec<Processor>,
    /// General conversion processors
    processors: Vec<Processor>,
    /// Node converters
    node_converters: Vec<NodeConverter>,
}

impl Conversion {
    pub fn from_extract(extract: Extract) -> Self {
        Conversion {
            extract,
            ..Default::default()
        }
    }

    pub fn add_preprocessor(&mut self, channel: Importance, fproc: ImplProcess) {
        self.preprocessors.push(Processor { channel, fproc })
    }

    pub fn add_processor(&mut self, channel: Importance, fproc: ImplProcess) {
        self.processors.push(Processor { channel, fproc })
    }

    pub fn add_node_converter(&mut self, select: SelectNode, fconvert: ImplConvertNode) {
        self.node_converters
            .push(NodeConverter { select, fconvert });
    }

    /// Perform conversion
    pub fn convert(&mut self) -> Converted {
        let mut partial = Partial::default();

        // preprocessors go first
        run_processor_batch(
            self,
            &mut partial,
            self.preprocessors
                .iter()
                .map(|procr| procr.make_runner())
                .collect(),
        );

        // do straight node conversion early, as the candidate inputs (in Extract) won't change.
        // considering only those with non-empty user types for direct conversion.
        let mut node_queue: HashMap<ExtractNodeId, &ExtractNode> = self
            .extract
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                if node.user_type.is_empty() {
                    None
                } else {
                    Some((*id, node))
                }
            })
            .collect();
        // set of node IDs that haven't been converted/consumed
        let mut candidate_ids: HashSet<ExtractNodeId> =
            self.extract.nodes.keys().cloned().collect();
        // to keep track of consumed (fully processed) nodes
        let mut consumed_nodes: HashSet<ExtractNodeId> = HashSet::new();
        // offer unconverted nodes to each converter
        for converter in &self.node_converters {
            for (node_id, node) in node_queue.extract_if(|_id, node| converter.select.selects(node))
            {
                let result = (converter.fconvert)(
                    self.extract.access(&node.id).unwrap(),
                    self,
                    &mut partial,
                );
                match result {
                    ConvertNodeResult::Pass => (),
                    ConvertNodeResult::Consume => {
                        consumed_nodes.insert(node_id);
                    }
                    ConvertNodeResult::ConsumeBranch => {
                        consumed_nodes.insert(node_id);
                        // prune descendants of converted node
                        consumed_nodes.extend(candidate_ids.extract_if(|candidate_id| {
                            self.extract.descends_from(*candidate_id, node_id)
                        }));
                    }
                }
            }
        }

        // general processors
        run_processor_batch(
            self,
            &mut partial,
            self.processors
                .iter()
                .map(|procr| procr.make_runner())
                .collect(),
        );

        // finalise output
        let resources = partial.map_elements;
        let chunks: Vec<Chunk> = partial.chunks.drain().map(|(_, chunk)| chunk).collect();
        Converted { resources, chunks }
    }
}

/// General conversion processor impl
pub type ImplProcess = fn(conversion: &Conversion, partial: &mut Partial) -> ProcessResult;

/// NodeConverter impl
pub type ImplConvertNode = fn(
    input: ExtractNodeAccess<'_>,
    conversion: &Conversion,
    partial: &mut Partial,
) -> ConvertNodeResult;

fn run_processor_batch(
    conversion: &mut Conversion,
    partial: &mut Partial,
    mut procrun: Vec<ProcessorRunner>,
) {
    // Run all the processors until they halt.
    while procrun.iter().any(ProcessorRunner::alive) {
        for runner in procrun.iter_mut().filter(|runner| runner.alive()) {
            if let Some(new_status) = runner.invoke(conversion, partial) {
                let failure = match runner.procr.channel {
                    Importance::Critical => !matches!(
                        new_status,
                        ProcessorRunnerStatus::Ok | ProcessorRunnerStatus::Success
                    ),
                    Importance::Conditional => matches!(
                        new_status,
                        ProcessorRunnerStatus::Stalled | ProcessorRunnerStatus::Failed
                    ),
                    Importance::Isolated => false,
                };
                if failure {
                    panic!(
                        "Conversion error {:?} processor status: {new_status:?}",
                        runner.procr.channel
                    );
                }
            }
        }
    }
}

/// Running processor status tracker
#[derive(Debug)]
pub struct ProcessorRunner {
    /// The processor to run
    procr: Processor,
    /// Most recent result code
    result: ProcessResult,
    /// Max process invocations remaining
    reps: u64,
    /// status code
    status: ProcessorRunnerStatus,
    /// set to true if/when the processor starts
    activated: bool,
}

impl ProcessorRunner {
    pub const REPS_MAX: u64 = 20;

    /// Invoke the processor if possible. Track status changes and if the runner status is changed,
    /// return the new status code as `Some`.
    pub fn invoke(
        &mut self,
        conversion: &mut Conversion,
        partial: &mut Partial,
        // output: &mut Converted,
    ) -> Option<ProcessorRunnerStatus> {
        if !self.alive() {
            return None;
        }

        let prev_status = self.status;
        if self.reps > 0 {
            self.reps -= 1;
            let result = (self.procr.fproc)(conversion, partial);
            self.result = result;
            match result {
                ProcessResult::Inactive => {
                    if self.activated {
                        // if had been active, treat inactivity as failure
                        self.status = ProcessorRunnerStatus::Failed;
                    } else {
                        // if never activated, adopt neutral inactive state
                        self.status = ProcessorRunnerStatus::Inactive;
                    }
                }
                ProcessResult::Ready => {}
                ProcessResult::Active => {
                    self.activated = true;
                }
                ProcessResult::Waiting => {
                    self.activated = true;
                }
                ProcessResult::Success => {
                    self.activated = true;
                    self.status = ProcessorRunnerStatus::Success;
                }
                ProcessResult::Failed => {
                    self.activated = true;
                    self.status = ProcessorRunnerStatus::Failed;
                }
            }
        } else if self.activated {
            // 'stalled' if active and reached rep limit
            self.status = ProcessorRunnerStatus::Stalled;
        } else {
            // 'inactive' if never activated and reached rep limit
            self.status = ProcessorRunnerStatus::Inactive;
        }

        if prev_status != self.status {
            Some(self.status)
        } else {
            None
        }
    }

    pub fn alive(&self) -> bool {
        self.status == ProcessorRunnerStatus::Ok && self.reps > 0
    }

    pub fn from_processor(procr: Processor) -> Self {
        ProcessorRunner {
            procr,
            result: ProcessResult::Ready,
            reps: Self::REPS_MAX,
            status: Default::default(),
            activated: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessorRunnerStatus {
    /// Normal process invocation
    #[default]
    Ok,
    /// Activated but failed to complete.
    Stalled,
    /// Processor reported inactive status -- do not invoke.
    Inactive,
    /// Process reported success
    Success,
    /// Process reported failure or something went wrong
    Failed,
}

#[derive(Debug, Clone)]
pub struct Processor {
    channel: Importance,
    fproc: ImplProcess,
}

impl Processor {
    pub fn make_runner(&self) -> ProcessorRunner {
        ProcessorRunner::from_processor(self.clone())
    }
}

#[derive(Debug, Default, Clone)]
pub enum Importance {
    /// The work is critical and it must be completed successfully. Failing this (including
    /// failure to start or if work stalls) will cause the overall conversion to fail.
    Critical,
    /// The processor activates conditionally depending on the context. Once activated, the
    /// processor must complete its work successfully. If the processor stalls or encounters an
    /// error, the error will propagate to the greater conversion context.
    #[default]
    Conditional,
    /// The processor is isolated from the rest of the conversion process. Activation state and
    /// completion status is ignored.
    Isolated,
}

/// Conversion process step result codes & processor status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessResult {
    /// Nothing to do. Cannot be activated in this conversion.
    Inactive,
    /// Idle & ready to work.
    Ready,
    /// Processor activated & work progressing.
    Active,
    /// Active process on hold while waiting for external block to clear.
    Waiting,
    /// Job completed successfully.
    Success,
    /// Encountered an unrecoverable error.
    Failed,
}

#[derive(Debug)]
pub struct NodeConverter {
    pub select: SelectNode,
    pub fconvert: ImplConvertNode,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectNode {
    All,
    UserType(&'static str),
    // IsObject,
    // IsLayer,
    // IsParent,
    AllOf(Vec<Self>),
    NoneOf(Vec<Self>),
}

impl SelectNode {
    pub fn selects(&self, node: &ExtractNode) -> bool {
        match self {
            SelectNode::All => true,
            SelectNode::UserType(class) => node.user_type == *class,
            SelectNode::AllOf(selectors) => selectors.iter().all(|selector| selector.selects(node)),
            SelectNode::NoneOf(selectors) => {
                !selectors.iter().any(|selector| selector.selects(node))
            }
        }
    }
}

/// One-shot node conversion result codes.
pub enum ConvertNodeResult {
    /// Converter did not activate, no work was performed.
    Pass,
    /// Indicates that the input node has been processed and prevents further processing of the
    /// same node by other converters.
    Consume,
    /// Indicates that the input node and its descendents have been fully processed, preventing
    /// further processing of the branch stemming from the input node.
    ConsumeBranch,
}
