use floaout::bub::{BubFnsBlock, BubMetadata, BubSampleKind, BubWriter};
use floaout::oao::{BubInOao, OaoMetadata, OaoWriter};
use floaout::wav::WavReader;
use floaout::LpcmKind;
use std::io::Result;

fn main() -> Result<()> {
    let bubs = vec![
        ("poly", "Poly", "-0.5-n/F 0 0 X<0.1 1.25/(0.5*PI)*E^(-(x^2+y^2)/0.5) 0.5+n/F 0 0 -0.1<X 1.25/(0.5*PI)*E^(-(x^2+y^2)/0.5)"),
        ("poly_first_half", "Poly first half", "0 0 0 0==0 0.5*sin(X+N/S)+0.5"),
        ("poly_sub", "Poly sub", "0 0 0 0==0 0.5*cos(X+N/S)+0.5"),
        ("bass1", "Bass 1", "0 0 0 0==0 0.5*cos(X+N/S)+0.5"),
        ("bass2", "Bass 2", "0 0 0 0==0 0.5*cos(X-N/S)+0.5"),
        ("pop", "Pop", "0 0 0 0==0 0.25*cos(X+N/S+PI/4)+0.75"),
        ("kick", "Kick", "0 0 0 X<0 0.85 0 0 0 0<=X 1"),
        ("crick", "Crick", "0 0 0 X<0 1 0 0 0 0<=X 0.9"),
        ("hit", "Hit", "0 0 0 0==0 1/(1+E^(-x-z+n/F)) 0 0 0 0==0 1/(1+E^(x+z+n/F))"),
    ];

    for bub in bubs {
        let wav_reader = WavReader::open(format!("{}.wav", bub.0))?;
        let frames = wav_reader.metadata.frames();
        let wav_frame_reader = unsafe { wav_reader.into_wav_frame_reader::<f32>() };
        let metadata = BubMetadata::new(
            frames,
            1,
            96000.0,
            LpcmKind::F32LE,
            BubSampleKind::Lpcm,
            String::from(bub.1),
        );
        // BubFnsBlock
        let mut samples = Vec::with_capacity(metadata.frames() as usize);
        for frame in wav_frame_reader {
            let frame = frame?;
            samples.push(frame.0[0]);
        }
        let bub_fns_block = BubFnsBlock::Lpcm {
            bub_fns: bub.2.as_bytes(),
            next_head_relative_frame: None,
            samples,
        };
        // Write
        let bub_writer = BubWriter::create(format!("{}.bub", bub.0), metadata)?;
        let mut bub_frame_writer = unsafe { bub_writer.into_bub_frame_writer::<f32>() };
        bub_frame_writer.write_head_to_less_than_next_head_or_ended(bub_fns_block)?;
    }

    // Create Oao
    let bubs_data = vec![
        ("poly", vec![1]),
        ("poly_first_half", vec![1]),
        ("poly_sub", vec![365715]),
        ("bass1", vec![1]),
        ("bass2", vec![1]),
        ("pop", vec![1]),
        ("kick", vec![1]),
        ("crick", vec![548572]),
        ("hit", vec![1]),
    ];
    let mut bubs = Vec::new();
    for bub in bubs_data {
        bubs.push(BubInOao {
            file_name: bub.0.into(),
            starting_frames: bub.1.into(),
        });
    }
    let oao_metadata = OaoMetadata::new(
        914285,
        96000.0,
        LpcmKind::F32LE,
        String::from("sample"),
        String::from("Kazuki Kurota"),
        bubs,
    );
    OaoWriter::create("sample.oao", oao_metadata)?;

    // 12 secs
    let metadata = BubMetadata::new(
        1_152_000,
        1,
        96000.0,
        LpcmKind::F32LE,
        BubSampleKind::default_expr(),
        "A440 and A880".into(),
    );
    let bub_writer = BubWriter::create("a440a880.bub", metadata)?;
    let mut bub_frame_writer = unsafe { bub_writer.into_bub_frame_writer::<f32>() };
    let bub_fns_block = BubFnsBlock::Expr {
        bub_fns: "0 0 0 0==0 0.5*sin(X+n/S)+0.5".as_bytes(),
        foot_relative_frame: 384_000,
        next_head_relative_frame: Some(576_001),
        expression: "sin(2*PI*440*n/S)".as_bytes(),
    };
    bub_frame_writer.write_head_to_less_than_next_head_or_ended(bub_fns_block)?;
    let bub_fns_block = BubFnsBlock::Expr {
        bub_fns: "0 0 0 0==0 0.5*sin(X-n/S)+0.5".as_bytes(),
        foot_relative_frame: 384_000,
        next_head_relative_frame: None,
        expression: "sin(2*PI*880*n/S)".as_bytes(),
    };
    bub_frame_writer.write_head_to_less_than_next_head_or_ended(bub_fns_block)?;

    Ok(())
}
