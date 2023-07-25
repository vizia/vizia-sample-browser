use std::cell::RefCell;

use usvg::{LineCap, LineJoin, NodeKind, PathSegment};

use vizia::{
    prelude::*,
    vg,
    vg::{Color, Paint, Path},
};

pub struct SvgDisplay {
    tree: RefCell<SvgTree>,
}

#[derive(Clone, Debug)]
enum SvgEvent {
    TreeChanged(SvgTree),
}

impl SvgDisplay {
    pub fn new<'a>(cx: &'a mut Context, icon: Icon) -> Handle<'a, Self> {
        let tree = SvgTree::from_str(
            &String::from_utf8(icon.to_bytes().to_vec()).unwrap(),
            &SvgOptions::default(),
        );

        Self { tree: RefCell::new(tree) }.build(cx, |cx| {})
    }
}

impl View for SvgDisplay {
    fn element(&self) -> Option<&'static str> {
        Some("svg")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        canvas.save();

        let b = cx.bounds();

        let mut tree = self.tree.borrow_mut();

        canvas.translate(b.x, b.y);
        canvas.scale(b.w / tree.size.0, b.h / tree.size.1);

        for (path, fill, stroke) in &mut tree.result {
            if let Some(fill) = fill {
                fill.set_anti_alias(true);
                canvas.fill_path(path, &fill);
            }

            if let Some(stroke) = stroke {
                stroke.set_anti_alias(true);
                canvas.stroke_path(path, &stroke);
            }
        }

        canvas.restore();
    }
}

#[derive(Lens, Clone, Default, Debug)]
pub struct SvgTree {
    result: Vec<(Path, Option<Paint>, Option<Paint>)>,
    size: (f32, f32),
}

pub type SvgOptions = usvg::Options;

impl SvgTree {
    pub fn from_str(text: &str, options: &SvgOptions) -> SvgTree {
        let tree = usvg::Tree::from_str(text, options).unwrap();
        let mut paths = vec![];

        for node in tree.root.descendants() {
            if let NodeKind::Path(svg_path) = &*node.borrow() {
                let mut path = Path::new();

                for command in svg_path.data.segments() {
                    match command {
                        PathSegment::MoveTo { x, y } => path.move_to(x as f32, y as f32),
                        PathSegment::LineTo { x, y } => path.line_to(x as f32, y as f32),
                        PathSegment::CurveTo { x1, y1, x2, y2, x, y } => path.bezier_to(
                            x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
                        ),
                        PathSegment::ClosePath => path.close(),
                    }
                }

                #[inline]
                fn to_femto_color(paint: &usvg::Paint) -> Option<Color> {
                    match paint {
                        usvg::Paint::Color(usvg::Color { red, green, blue }) => {
                            Some(Color::rgb(*red, *green, *blue))
                        }
                        _ => None,
                    }
                }

                let mut fill = svg_path
                    .fill
                    .as_ref()
                    .and_then(|fill| to_femto_color(&fill.paint))
                    .map(|v| Paint::color(v));

                let mut stroke = svg_path.stroke.as_ref().and_then(|stroke| {
                    to_femto_color(&stroke.paint).map(|paint| {
                        let mut stroke_paint = Paint::color(paint);
                        stroke_paint.set_line_width(stroke.width.get() as f32);

                        let line_cap: vg::LineCap = match stroke.linecap {
                            LineCap::Butt => vg::LineCap::Butt,
                            LineCap::Square => vg::LineCap::Square,
                            LineCap::Round => vg::LineCap::Round,
                        };

                        let line_join: vg::LineJoin = match stroke.linejoin {
                            LineJoin::Bevel => vg::LineJoin::Bevel,
                            LineJoin::Miter => vg::LineJoin::Miter,
                            LineJoin::Round => vg::LineJoin::Round,
                        };

                        stroke_paint.set_line_cap(line_cap);
                        stroke_paint.set_line_join(line_join);

                        stroke_paint.set_miter_limit(stroke.miterlimit.get() as f32);
                        stroke_paint
                    })
                });

                // if let Some(override_color) = override_color {
                //     if let Some(paint) = &mut fill {
                //         paint.set_color(override_color.into());
                //     }
                //     if let Some(paint) = &mut stroke {
                //         paint.set_color(override_color.into());
                //     }
                // }

                paths.push((path, fill, stroke));
            }
        }

        Self { result: paths, size: (tree.size.width() as f32, tree.size.height() as f32) }
    }
}

const ICON_ARROW_LEFT: &[u8] = include_bytes!("../icons/Icon=Arrow Left.svg");
const ICON_ARROW_RIGHT: &[u8] = include_bytes!("../icons/Icon=Arrow Right.svg");
const ICON_AUDIO_DISABLED: &[u8] = include_bytes!("../icons/Icon=Audio Disabled.svg");
const ICON_BOLT: &[u8] = include_bytes!("../icons/Icon=Bolt.svg");
const ICON_CHECK: &[u8] = include_bytes!("../icons/Icon=Check.svg");
const ICON_CHEVRON_DOWN: &[u8] = include_bytes!("../icons/Icon=Chevron Down.svg");
const ICON_CHEVRON_LEFT: &[u8] = include_bytes!("../icons/Icon=Chevron Left.svg");
const ICON_CHEVRON_RIGHT: &[u8] = include_bytes!("../icons/Icon=Chevron Right.svg");
const ICON_CHEVRON_UP: &[u8] = include_bytes!("../icons/Icon=Chevron Up.svg");
const ICON_CLOCK: &[u8] = include_bytes!("../icons/Icon=Clock.svg");
const ICON_COLORPICKER: &[u8] = include_bytes!("../icons/Icon=Colorpicker.svg");
const ICON_CURSOR: &[u8] = include_bytes!("../icons/Icon=Cursor.svg");
const ICON_EYE_CLOSED: &[u8] = include_bytes!("../icons/Icon=Eye Closed.svg");
const ICON_EYE_EDIT: &[u8] = include_bytes!("../icons/Icon=Eye Edit.svg");
const ICON_EYE: &[u8] = include_bytes!("../icons/Icon=Eye.svg");
const ICON_FILTER: &[u8] = include_bytes!("../icons/Icon=Filter.svg");
const ICON_FOLDER_OPEN: &[u8] = include_bytes!("../icons/Icon=Folder Open.svg");
const ICON_FOLDER: &[u8] = include_bytes!("../icons/Icon=Folder.svg");
const ICON_FOLLOW_LINE: &[u8] = include_bytes!("../icons/Icon=Follow Line.svg");
const ICON_GRID: &[u8] = include_bytes!("../icons/Icon=Grid.svg");
const ICON_HEADSET: &[u8] = include_bytes!("../icons/Icon=Headset.svg");
const ICON_MAGNET: &[u8] = include_bytes!("../icons/Icon=Magnet.svg");
const ICON_MENU: &[u8] = include_bytes!("../icons/Icon=Menu.svg");
const ICON_PAUSE: &[u8] = include_bytes!("../icons/Icon=Pause.svg");
const ICON_PENCIL: &[u8] = include_bytes!("../icons/Icon=Pencil.svg");
const ICON_PLAY: &[u8] = include_bytes!("../icons/Icon=Play.svg");
const ICON_POINTER: &[u8] = include_bytes!("../icons/Icon=Pointer.svg");
const ICON_SCISSORS: &[u8] = include_bytes!("../icons/Icon=Scissors.svg");
const ICON_SEARCH: &[u8] = include_bytes!("../icons/Icon=Search.svg");
const ICON_SETTINGS: &[u8] = include_bytes!("../icons/Icon=Settings.svg");
const ICON_SMALL_CROSS: &[u8] = include_bytes!("../icons/Icon=Small Cross.svg");
const ICON_SMALL_SQUARE: &[u8] = include_bytes!("../icons/Icon=Small Square.svg");
const ICON_STAR: &[u8] = include_bytes!("../icons/Icon=Star.svg");
const ICON_STOP: &[u8] = include_bytes!("../icons/Icon=Stop.svg");
const ICON_TEXT: &[u8] = include_bytes!("../icons/Icon=Text.svg");
const ICON_TOOL: &[u8] = include_bytes!("../icons/Icon=Tool.svg");
const ICON_TRASH_CAN: &[u8] = include_bytes!("../icons/Icon=Trash Can.svg");
const ICON_WAND: &[u8] = include_bytes!("../icons/Icon=Wand.svg");
const ICON_WEIGHT: &[u8] = include_bytes!("../icons/Icon=Weight.svg");

pub enum Icon {
    ARROW_LEFT,
    ARROW_RIGHT,
    AUDIO_DISABLED,
    BOLT,
    CHECK,
    CHEVRON_DOWN,
    CHEVRON_LEFT,
    CHEVRON_RIGHT,
    CHEVRON_UP,
    CLOCK,
    COLORPICKER,
    CURSOR,
    EYE_CLOSED,
    EYE_EDIT,
    EYE,
    FILTER,
    FOLDER_OPEN,
    FOLDER,
    FOLLOW_LINE,
    GRID,
    HEADSET,
    MAGNET,
    MENU,
    PAUSE,
    PENCIL,
    PLAY,
    POINTER,
    SCISSORS,
    SEARCH,
    SETTINGS,
    SMALL_CROSS,
    SMALL_SQUARE,
    STAR,
    STOP,
    TEXT,
    TOOL,
    TRASH_CAN,
    WAND,
    WEIGHT,
}

impl Icon {
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            Icon::ARROW_LEFT => ICON_ARROW_LEFT,
            Icon::ARROW_RIGHT => ICON_ARROW_RIGHT,
            Icon::AUDIO_DISABLED => ICON_AUDIO_DISABLED,
            Icon::BOLT => ICON_BOLT,
            Icon::CHECK => ICON_CHECK,
            Icon::CHEVRON_DOWN => ICON_CHEVRON_DOWN,
            Icon::CHEVRON_LEFT => ICON_CHEVRON_LEFT,
            Icon::CHEVRON_RIGHT => ICON_CHEVRON_RIGHT,
            Icon::CHEVRON_UP => ICON_CHEVRON_UP,
            Icon::CLOCK => ICON_CLOCK,
            Icon::COLORPICKER => ICON_COLORPICKER,
            Icon::CURSOR => ICON_CURSOR,
            Icon::EYE_CLOSED => ICON_EYE_CLOSED,
            Icon::EYE_EDIT => ICON_EYE_EDIT,
            Icon::EYE => ICON_EYE,
            Icon::FILTER => ICON_FILTER,
            Icon::FOLDER_OPEN => ICON_FOLDER_OPEN,
            Icon::FOLDER => ICON_FOLDER,
            Icon::FOLLOW_LINE => ICON_FOLLOW_LINE,
            Icon::GRID => ICON_GRID,
            Icon::HEADSET => ICON_HEADSET,
            Icon::MAGNET => ICON_MAGNET,
            Icon::MENU => ICON_MENU,
            Icon::PAUSE => ICON_PAUSE,
            Icon::PENCIL => ICON_PENCIL,
            Icon::PLAY => ICON_PLAY,
            Icon::POINTER => ICON_POINTER,
            Icon::SCISSORS => ICON_SCISSORS,
            Icon::SEARCH => ICON_SEARCH,
            Icon::SETTINGS => ICON_SETTINGS,
            Icon::SMALL_CROSS => ICON_SMALL_CROSS,
            Icon::SMALL_SQUARE => ICON_SMALL_SQUARE,
            Icon::STAR => ICON_STAR,
            Icon::STOP => ICON_STOP,
            Icon::TEXT => ICON_TEXT,
            Icon::TOOL => ICON_TOOL,
            Icon::TRASH_CAN => ICON_TRASH_CAN,
            Icon::WAND => ICON_WAND,
            Icon::WEIGHT => ICON_WEIGHT,
        }
    }
}
