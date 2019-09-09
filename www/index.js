import {Point, Polygon, init_panic_hook} from "triangulation";

init_panic_hook();

let d3 = require("d3");
let tri_text = d3.select("#textarea")
    .append("span");

let svg = d3.select("body").append("svg")
    .attr("width", window.innerWidth)
    .attr("height", window.innerHeight); //TODO fit to window and scale on change


let points = [];
let draw_line = (x1, y1, x2, y2) => {
    svg.append("line")
        .attr("x1", x1)
        .attr("y1", y1)
        .attr("x2", x2)
        .attr("y2", y2)
        .style("stroke", "rgb(255,0,0)") // move style later
        .style("stroke-width", 2);
};
let connect_polygon = (x, y) => {
    if (points.length > 0) {
        // try make origin bottom left better
        draw_line(points[points.length-2], -points[points.length-1] - window.innerHeight, x, y)
    }
};

let delete_polygon =  () => {
    d3.selectAll("circle")
        .remove();
    d3.selectAll("line")
        .remove();
    points = []
};

let undo_move = () => {
    d3.select("svg>circle:last-child").remove();
    d3.select("svg>line:last-child").remove();
    if (polygon_done) {
        polygon_done = false;
        return;
    }
    points.pop();
    points.pop();
    console.log(points);
};
d3.select("body")
    .on("keydown", function() {
        if (d3.event.keyCode == 8) {
            delete_polygon();
        }
        else if (d3.event.ctrlKey && d3.event.keyCode == 90) {
            undo_move();
        }
    });

let polygon_done = false;

svg.on("click", () => {
    console.log(points);
    if (d3.event.defaultPrevented) {
        return
    }
    if (polygon_done) {
        delete_polygon();
        polygon_done = false;
        return
    }
    let [x, y] = [d3.event.pageX, d3.event.pageY];
    connect_polygon(x, y);

    points.push(x);
    points.push(-(y + window.innerHeight));

    // TODO move c_scale
    let c_scale = 1.5;
    let origin_x = x - c_scale * x;
    let origin_y = y - c_scale * y;

    svg  // For new circle, go through the update process
        .append("circle")
        .on("mouseover", function()  {
            d3.select(this).attr("transform", `matrix(${c_scale}, 0, 0, ${c_scale}, ${origin_x}, ${origin_y})`);
        })
        .on("mouseout", function() {
            d3.select(this).attr("transform", "");
        })
        .attr("cx", x)
        .attr("cy", y)
        .attr("r", 10)
        .on("click", () => {
            d3.event.preventDefault();
            connect_polygon(x, y);
            let poly = Polygon.from_slice(new Float32Array(points));
            poly.ccw_sort();
            tri_text.text("Number of triangulations: " + poly.nb_triangulations());
            polygon_done = true;
        })

});


