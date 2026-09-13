fn main() {
    for id in [394u16, 395, 651, 652] {
        println!(
            "{id}: name={} state_block={} default_state={}",
            vc_blocks::blocks::name(id),
            vc_blocks::blocks::state_block(id),
            vc_blocks::blocks::default_state(id)
        );
    }
}
