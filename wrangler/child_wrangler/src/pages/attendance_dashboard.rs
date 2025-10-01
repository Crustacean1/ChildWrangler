use std::f32::consts::PI;

use leptos::logging::log;
use leptos::prelude::*;

#[component]
pub fn Chart(padding: i32, series: Vec<(String, i32)>) -> impl IntoView {
    let range = (360 - series.len() * padding as usize) as f32 / 360 as f32;
    let scalar = 2.0 * PI * (padding as f32 / 360 as f32);
    let total = series.iter().map(|(_, s)| s).sum::<i32>() as f32;
    let sizes = series
        .iter()
        .map(|(_, s)| (*s as f32) / total)
        .scan(0.0, |state, x| {
            *state += x;
            Some(*state * 2.0 * PI * range)
        })
        .collect::<Vec<_>>();

    let radius = 64.0;
    let colors = vec!["#ff0000", "#00ff00", "#0000ff", "#ffff00"];

    view! {

            <svg width="256" height="256" viewBox="-128 -128 256 256">
        { [0.0]
        .iter()
        .chain(sizes.iter())
        .zip(sizes.iter())
        .enumerate()
        .map(|(i,(start, end))| {
                log!("{:?} {:?}", start,end);
                let start = start + i  as f32* scalar;
                let end = end + i as f32 * scalar;
            let path = format!(
                "M {} {} A {} {} 0 {} 1 {} {}",
                (start).cos() * radius,
                (start).sin() * radius,
                radius,
                radius,
                (end - start > PI) as i32,
                (end).cos() * radius,
                (end).sin() * radius
            );
            view!{
                <path d={path} stroke=colors[i] stroke-width="8" fill="none" stroke-linecap="round"/>
            }
        })
        .collect::<Vec<_>>()}
            <text x="0" y="0" text-anchor="middle" fill="white" dominant-baseline="middle" font-size="2em">123</text>
            </svg>
    }
}

#[component]
pub fn AttendanceDashboard() -> impl IntoView {
    view! {
        <div>
        <div class="padded vertical rounded background-2">
        <h2 class="h2">Obiad</h2>
        <div class="horizontal gap align-center">
        {move || view!{
        <Chart padding={10} series=vec![(String::from("Entity 1"), 15), (String::from("Entity2"), 25),
            (String::from("Entity3"), 35)]/>
        }}
        <div class="vertical gap align-start justify-center">
            <div class="rounded-decoration">Obecni</div>
            <div class="rounded-decoration">Odmówieni</div>
            <div class="rounded-decoration">Anulowani</div>
            <div class="rounded-decoration">Alergicy</div>
            </div>
        </div>
        </div>

        </div>
    }
}
