/*
 * Voronoi sample code from https://www.redblobgames.com/x/2022-voronoi-maps-tutorial/
 * Open source, under the Apache v2.0 license <http://www.apache.org/licenses/LICENSE-2.0.html>
 *
 * GENERATED FILE - see index.org for original
 */

const GRIDSIZE = 25;
const JITTER = 0.5;
let points = generateJitteredGridPoints(GRIDSIZE, JITTER);

function generateJitteredGridPoints(gridsize, jitter) {
    let points = [];
    for (let x = 0; x <= gridsize; x++) {
        for (let y = 0; y <= gridsize; y++) {
            points.push({x: x + jitter * (Math.random() - Math.random()),
                y: y + jitter * (Math.random() - Math.random())});
        }
    }
    return points;
}

function drawPoints(canvas, gridsize, points, radius=0.1) {
    let ctx = canvas.getContext('2d');
    ctx.save();
    ctx.scale(canvas.width / gridsize, canvas.height / gridsize);
    ctx.fillStyle = "hsl(0 50% 50%)";
    for (let {x, y} of points) {
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, 2*Math.PI);
        ctx.fill();
    }
    ctx.restore();
}

drawPoints(document.getElementById("diagram-points"), GRIDSIZE, points);

let delaunay = Delaunator.from(points, loc => loc.x, loc => loc.y);

function calculateCentroids(points, delaunay) {
    const numTriangles = delaunay.halfedges.length / 3;
    let centroids = [];
    for (let t = 0; t < numTriangles; t++) {
        let sumOfX = 0, sumOfY = 0;
        for (let i = 0; i < 3; i++) {
            let s = 3*t + i;
            let p = points[delaunay.triangles[s]];
            sumOfX += p.x;
            sumOfY += p.y;
        }
        centroids[t] = {x: sumOfX / 3, y: sumOfY / 3};
    }
    return centroids;
}

let map = {
    points,
    gridsize: GRIDSIZE,
    numRegions: points.length,
    numTriangles: delaunay.halfedges.length / 3,
    numEdges: delaunay.halfedges.length,
    halfedges: delaunay.halfedges,
    triangles: delaunay.triangles,
    centers: calculateCentroids(points, delaunay)
};

function triangleOfEdge(e)  { return Math.floor(e / 3); }
function nextHalfedge(e) { return (e % 3 === 2) ? e - 2 : e + 1; }

function drawCellBoundaries(canvas, map) {
    let {points, centers, halfedges, triangles, numEdges} = map;
    let ctx = canvas.getContext('2d');
    ctx.save();
    ctx.scale(canvas.width / map.gridsize, canvas.height / map.gridsize);
    ctx.lineWidth = 0.02;
    ctx.strokeStyle = "black";
    for (let e = 0; e < numEdges; e++) {
        if (e < halfedges[e]) {
            const p = centers[triangleOfEdge(e)];
            const q = centers[triangleOfEdge(halfedges[e])];
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(q.x, q.y);
            ctx.stroke();
        }
    }
    ctx.restore();
}
drawPoints(document.getElementById("diagram-boundaries"), GRIDSIZE, points, 0.07);
drawCellBoundaries(document.getElementById("diagram-boundaries"), map);

const WAVELENGTH = 0.5;
function assignElevation(map) {
    const noise = new SimplexNoise();
    let {points, numRegions} = map;
    let elevation = [];
    for (let r = 0; r < numRegions; r++) {
        let nx = points[r].x / map.gridsize - 1/2,
            ny = points[r].y / map.gridsize - 1/2;
        // start with noise; decide how many octaves you want and what their amplitudes are
        // see https://www.redblobgames.com/maps/terrain-from-noise/#elevation
        elevation[r] = (1/2
            + noise.noise2D(nx / WAVELENGTH, ny / WAVELENGTH) / 2
            + noise.noise2D(2 * nx / WAVELENGTH, 2 * ny / WAVELENGTH) / 3
        );
        // modify noise to make islands:
        let d = 2 * Math.max(Math.abs(nx), Math.abs(ny)); // should be 0-1
        elevation[r] = (1 + elevation[r] - d) / 2;
    }
    return elevation;
}

map.elevation = assignElevation(map);

function edgesAroundPoint(halfedges, start) {
    const result = [];
    let incoming = start;
    do {
        result.push(incoming);
        const outgoing = nextHalfedge(incoming);
        incoming = halfedges[outgoing];
    } while (incoming !== -1 && incoming !== start);
    return result;
}

function drawCellColors(canvas, map, colorFn) {
    let ctx = canvas.getContext('2d');
    ctx.save();
    ctx.scale(canvas.width / map.gridsize, canvas.height / map.gridsize);
    let seen = new Set();  // of region ids
    let {triangles, numEdges, centers} = map;
    for (let e = 0; e < numEdges; e++) {
        const r = triangles[nextHalfedge(e)];
        if (!seen.has(r)) {
            seen.add(r);
            let vertices = edgesAroundPoint(map.halfedges, e)
                .map(e => centers[triangleOfEdge(e)]);
            ctx.fillStyle = colorFn(r);
            ctx.beginPath();
            ctx.moveTo(vertices[0].x, vertices[0].y);
            for (let i = 1; i < vertices.length; i++) {
                ctx.lineTo(vertices[i].x, vertices[i].y);
            }
            ctx.fill();
        }
    }
    ctx.restore();
}

drawCellColors(
    document.getElementById("diagram-cell-elevations"),
    map,
    r => map.elevation[r] < 0.5? "hsl(240, 30%, 50%)" : "hsl(90, 20%, 50%)"
);

function assignMoisture(map) {
    const noise = new SimplexNoise();
    let {points, numRegions} = map;
    let moisture = [];
    for (let r = 0; r < numRegions; r++) {
        let nx = points[r].x / map.gridsize - 1/2,
            ny = points[r].y / map.gridsize - 1/2;
        moisture[r] = (1 + noise.noise2D(nx / WAVELENGTH, ny / WAVELENGTH)) / 2;
    }
    return moisture;
}

map.moisture = assignMoisture(map);

function biomeColor(map, r) {
    let e = (map.elevation[r] - 0.5) * 2, // rescale 0:1 to -1:+1
        m = map.moisture[r];
    if (e < 0.0) {
        r = 48 + 48*e;
        g = 64 + 64*e;
        b = 127 + 127*e;
    } else {
        e = e**4; // tweak for better coloring
        r = 210 - 100 * m;
        g = 185 - 45 * m;
        b = 139 - 45 * m;
        r = 255 * e + r * (1-e),
            g = 255 * e + g * (1-e),
            b = 255 * e + b * (1-e);
    }
    return `rgb(${r|0}, ${g|0}, ${b|0})`;
}

drawCellColors(
    document.getElementById("diagram-cell-biomes"),
    map,
    r => biomeColor(map, r)
);

function biomeColorDiscrete(map, r) {
    let e = 2.0 * (map.elevation[r] - 0.5); // convert 0:1 to -1:+1
    let m = map.moisture[r] ** 2; // tweak as needed

    if (e < 0.0) return "#44447a"; // ocean

    if (e > 0.6) {
        if (m < 0.2) return "#888888"; // barren rock
        if (m < 0.5) return "#bbbbaa"; // tundra
        else         return "#dddde4"; // glacier
    }

    if (e > 0.4) {
        if (m < 0.33) return "#c9d29b"; // temperate desert
        if (m < 0.66) return "#889977"; // shrubland
        else          return "#99aa77"; // taiga
    }

    if (e > 0.2) {
        if (m < 0.16) return "#c9d29b"; // temperate desert
        if (m < 0.50) return "#88aa55"; // grassland
        if (m < 0.83) return "#679459"; // temperate deciduous forest
        else          return "#448855"; // temperate rain forest
    }

    if (m < 0.16) return "#d2b98b"; // subtropical desert
    if (m < 0.33) return "#88aa55"; // grassland
    if (m < 0.66) return "#559944"; // tropical seasonal forest
    else          return "#337755"; // tropical rain forest
}

drawCellColors(
    document.getElementById("diagram-cell-biomes-discrete"),
    map,
    r => biomeColorDiscrete(map, r)
);

function assignDownslope(map) {
    let {elevation, halfedges, triangles} = map;
    let downslope = Array.from({length: elevation.length}, () => undefined); // undefined if we never calculated, null if it's a local minimum
    for (let incoming1 = 0; incoming1 < triangles.length; incoming1++) {
        let outgoing1 = halfedges[incoming1];
        if (outgoing1 < 0) continue; // on convex hull, ignore
        let r1 = triangles[outgoing1];
        let bestElevation = elevation[r1];
        let bestEdge = null;
        for (let incoming2 of edgesAroundPoint(halfedges, incoming1)) {
            let r2 = triangles[incoming2];
            if (elevation[r2] < bestElevation) {
                bestElevation = elevation[r2];
                bestEdge = incoming2; // this seems backwards, r2->r1, but it's convenient this direction later
            }
        }
        downslope[r1] = bestEdge;
    }
    return downslope;
}

map.downslope = assignDownslope(map);

function drawDownslope(canvas, map) {
    let {points, downslope, halfedges, triangles} = map;
    let ctx = canvas.getContext('2d');
    ctx.save();
    ctx.scale(canvas.width / map.gridsize, canvas.height / map.gridsize);
    ctx.lineWidth = 0.02;
    ctx.strokeStyle = "blue";
    ctx.fillStyle = "blue";
    for (let r1 = 0; r1 < points.length; r1++) {
        if (downslope[r1] === undefined || downslope[r1] === null) continue; // no outflow
        let r2 = triangles[downslope[r1]];
        let p = points[r1];
        let q = points[r2];
        ctx.fillRect(p.x-0.03, p.y-0.03, 0.06, 0.06);
        ctx.beginPath();
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(0.5 * (p.x + q.x), 0.5 * (p.y + q.y));
        ctx.stroke();
    }
    ctx.restore();
}

drawCellColors(document.getElementById("diagram-downslope"), map, r => `hsl(100 ${map.elevation[r] < 0.5? 0 : 20}% ${map.elevation[r] * 100}%)`);
drawDownslope(document.getElementById("diagram-downslope"), map);

drawCellColors(
    document.getElementById("diagram-rainfall"),
    map,
    r => map.elevation[r] < 0.5? `hsl(0 0% 70%)` : `hsl(220 ${map.moisture[r] * 100}% 50%)`
);

function assignRiverFlow(map) {
    let {elevation, moisture, downslope, triangles} = map;
    let regions = Array.from({length: elevation.length}, (_, r) => r);
    regions.sort((r1, r2) => elevation[r2] - elevation[r1]); // sort higher elevations first

    let flow = Array.from({length: elevation.length}, () => 0);
    for (let r of regions) {
        if (elevation[r] < 0.5) continue; // skip oceans
        flow[r] += moisture[r]; // rainfall

        let incomingEdge = downslope[r];
        if (incomingEdge === null) continue; // this is the final point
        let outgoingRegion = triangles[incomingEdge];
        flow[outgoingRegion] += flow[r];
    }
    return flow;
}

map.flow = assignRiverFlow(map);

function drawRivers(canvas, map, threshold) {
    let {points, flow, downslope, triangles} = map;
    let ctx = canvas.getContext('2d');
    ctx.save();
    ctx.scale(canvas.width / map.gridsize, canvas.height / map.gridsize);
    ctx.strokeStyle = "rgb(48, 64, 127)";
    ctx.lineCap = 'round';
    for (let r1 = 0; r1 < points.length; r1++) {
        if (downslope[r1] === undefined || downslope[r1] === null) continue; // no outflow
        if (flow[r1] < threshold) continue; // don't draw small rivers
        let r2 = triangles[downslope[r1]];
        let p = points[r1];
        let q = points[r2];
        ctx.lineWidth = 0.05 * Math.sqrt(flow[r1]);
        ctx.beginPath();
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(q.x, q.y);
        ctx.stroke();
    }
    ctx.restore();
}

drawCellColors(document.getElementById("diagram-rivers"), map, r => biomeColor(map, r));
drawRivers(document.getElementById("diagram-rivers"), map, 0.01);

function drawRiversWithThreshold() {
    let logOfThreshold = document.getElementById("river-log-threshold").valueAsNumber ?? 0.1;
    let threshold = Math.pow(10, logOfThreshold);
    drawCellColors(document.getElementById("diagram-rivers-threshold"), map, r => biomeColor(map, r));
    drawRivers(document.getElementById("diagram-rivers-threshold"), map, threshold);
}
drawRiversWithThreshold();
document.getElementById("river-log-threshold").addEventListener('input', drawRiversWithThreshold);

// draw the same thing at the top of the page, but with boundary points
function diagramTopOfPage() {
    let gridsize = GRIDSIZE*3;
    let points = generateJitteredGridPoints(gridsize, JITTER);
    points.push({x: -10, y: gridsize/2});
    points.push({x: gridsize+10, y: gridsize/2});
    points.push({y: -10, x: gridsize/2});
    points.push({y: gridsize+10, x: gridsize/2});
    points.push({x: -10, y: -10});
    points.push({x: gridsize+10, y: gridsize+10});
    points.push({y: -10, x: gridsize+10});
    points.push({y: gridsize+10, x: -10});

    let delaunay = Delaunator.from(points, loc => loc.x, loc => loc.y);
    let map = {
        points,
        gridsize,
        numRegions: points.length,
        numTriangles: delaunay.halfedges.length / 3,
        numEdges: delaunay.halfedges.length,
        halfedges: delaunay.halfedges,
        triangles: delaunay.triangles,
        centers: calculateCentroids(points, delaunay)
    };
    map.elevation = assignElevation(map);
    map.moisture = assignMoisture(map);
    map.downslope = assignDownslope(map);
    map.flow = assignRiverFlow(map);

    let el = document.getElementById("map");

    el.getContext('2d').clearRect(0, 0, el.width, el.height);
    drawCellColors(el, map, r => biomeColor(map, r));
    drawRivers(el, map, 5.0);
}
diagramTopOfPage();
document.getElementById("map").addEventListener('click', diagramTopOfPage);
