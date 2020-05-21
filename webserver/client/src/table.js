"use strict"
import Mousetrap from "mousetrap";

import { SpectralSequenceChart } from "chart/sseq/SpectralSequenceChart.js";
window.SpectralSequenceChart = SpectralSequenceChart;

import ReconnectingWebSocket from 'reconnecting-websocket';

import { UIElement } from "chart/interface/UIElement.js";
import { Display } from "chart/interface/Display.js";
import { AxesElement } from "chart/interface/Axes.js";
import { GridElement } from "chart/interface/GridElement.js";
import { ChartElement } from "chart/interface/ChartElement.js";
import { ClassHighlighter } from "chart/interface/ClassHighlighter";
import { SseqPageIndicator } from "chart/interface/SseqPageIndicator.js";
import { Tooltip } from "chart/interface/Tooltip.js";



import { Panel } from "chart/interface/Panel.js";
import { Matrix } from "chart/interface/Matrix.js";
import { KatexExprElement } from "chart/interface/KatexExprElement.js";
import { SseqSocketListener } from "chart/SseqSocketListener.js";
import { Popup } from "chart/interface/Popup.js";
import { sleep, promiseFromDomEvent } from "chart/interface/utils.js";

window.SseqSocketListener = SseqSocketListener;


window.main = main;

function main(display, socket_address){
    // let matrix = document.querySelector("sseq-matrix");
    // matrix.value = [
    //     [0,0,0],
    //     [1, 0, 1],
    //     [1,1,1]
    // ];
    // matrix.addEventListener("matrix-click", (e) => {
    //     let row = e.detail.row_idx;
    //     if(matrix.selectedRows.includes(row)){
    //         matrix.selectedRows = [];
    //     } else {
    //         matrix.selectedRows = [e.detail.row_idx];
    //     }
    // });

    // Mousetrap(matrix).bind("escape", () =>{
    //     matrix.selectedRows = [];
    // });


    let ws = new ReconnectingWebSocket(socket_address);
    window.socket_listener = new SseqSocketListener(ws);
    socket_listener.attachDisplay(display);
    Mousetrap.bind("left", display.previousPage)
    Mousetrap.bind("right", display.nextPage)
    Mousetrap.bind("t", () => {
        socket_listener.send("console.take", {});
    });


    function productMouseover(e){
        console.log(e);
    }

    function productMouseout(e){
        console.log(e);
    }

    let names;
    let product_info;
    let matrix;
    let selected_bidegree;
    socket_listener.add_message_handler("interact.product_info", function(cmd, args, kwargs){
        let sseq = display.querySelector("sseq-chart");
        names = kwargs.names;
        product_info = kwargs.product_info;
        matrix = kwargs.matrix;
        let result = [];
        for(let [[in1,name1, mono1], [in2, name2, mono2], out, preimage, possible_name] of product_info){
            let name_str = "";
            if(possible_name){
                name_str = `{}= ${possible_name}`
            }
            result.push([`${name1} \\cdot ${name2} = ${JSON.stringify(out)}`, name_str]);
        }
        let sidebar = document.querySelector("sseq-panel");
        let class_html = "";
        let product_html = "";
        let matrix_html = "";
        class_html = `
            <h5>
            Classes in (${selected_bidegree.join(", ")})
            </h5>
            <p style="align-self: center;">
                ${names.map(e => `<katex-expr class="name">${e}</katex-expr>`)
                        .join(`, <span style="padding-right:6pt; display:inline-block;"></span>`)}
            </p>
        `;        
        if(result.length > 0){
            product_html = `
                <h5 style="">
                    Products
                </h5>
                <div class="product-list" style="align-self: center; width: max-content; overflow: hidden;">
                    <table><tbody>
                        ${result.map(([e, n]) => `
                            <tr class="product-item">
                                <td align='right'><katex-expr>${e}</katex-expr></td>
                                <td><katex-expr>${n}</katex-expr></td>
                            </tr>
                        `).join("")}
                    </tbody></table>
                </div>            
            `;

            matrix_html = `
                <h5 style="margin-top:12pt;">Matrix:</h5>
                <sseq-matrix type="display" style="align-self:center;"></sseq-matrix>
            `;
        }
        sidebar.querySelector("#product-info-classes").innerHTML = class_html;
        sidebar.querySelector("#product-info-products").innerHTML = product_html;
        sidebar.querySelector("#product-info-matrix").innerHTML = matrix_html;

        sidebar.querySelectorAll(".product-item").forEach((e, idx) => {
            e.addEventListener("click",  () => {
                productItemClick(idx);
                // socket_listener.send("interact.click_product", {"bidegree" : sseq._selected_bidegree, "idx" : idx});
            });
        });
        sidebar.querySelector("sseq-matrix").value = matrix;
        sidebar.querySelector("sseq-matrix").labels = names;
        sidebar.displayChildren("#product-info");

        // div.style.display = "flex";
        // div.style.flexDirection = "column";
        // div.style.height = "90%";
    })
    

    async function productItemClick(item_idx){
        let sseq = display.querySelector("sseq-chart").sseq;
        let jsoned_matrix = matrix.map(JSON.stringify);
        let product_data = product_info[item_idx];
        let [[in1, name1, _nm1], [in2, name2, _nm2], out, out_vec, out_name] = product_data;
        let index = jsoned_matrix.indexOf(JSON.stringify(out));
        let inbasis = index != -1;
        let popup = document.querySelector("sseq-popup");
        let popup_header = popup.querySelector("[slot=header]");
        let popup_body = popup.querySelector("[slot=body]");
        let bidegree = selected_bidegree;
        let highlightClasses = [sseq.class_by_index(...in1), sseq.class_by_index(...in2)];
        if(inbasis){
            let out_tuple = [...bidegree, index];
            let nameWord = out_name ? "Rename" : "Name";
            popup_header.innerText = `${nameWord} class?`;
            popup_body.innerHTML = `
                <p>${nameWord} class (${out_tuple.join(", ")}) as <katex-expr>${name1}\\cdot ${name2}</katex-expr>?</p>
                ${out_name ? `<p>Current name is <katex-expr>${out_name}</katex-expr>.</p>` : ``}
            `;
            highlightClasses.push(sseq.class_by_index(...out_tuple));
        } else {
            popup_header.innerText = "Update basis?";
            popup_body.innerHTML = `
                Select a basis vector to replace:
                <p><sseq-matrix type=display></sseq-matrix></p>
            `;
            await sleep(10);
            let matrix_elt = popup_body.querySelector("sseq-matrix");
            matrix_elt.value = matrix;
            matrix_elt.addEventListener("matrix-click", (e) => {
                let row = e.detail.row_idx;
                if(matrix_elt.selectedRows.includes(row)){
                    matrix_elt.selectedRows = [];
                } else {
                    matrix_elt.selectedRows = [e.detail.row_idx];
                }
            });
            
        }
        document.querySelector("sseq-popup").show();
        let class_highlighter = document.querySelector("sseq-class-highlighter");
        let result = class_highlighter.clear();
        class_highlighter.fire(highlightClasses, 0.8);
    }
    
    display.addEventListener("click", function(e){
        let sseq = display.querySelector("sseq-chart").sseq;
        let new_bidegree = e.detail[0].mouseover_bidegree;
        if(
            selected_bidegree
            && new_bidegree[0] == selected_bidegree[0] 
            && new_bidegree[1] == selected_bidegree[1]
        ){
            return;
        }
        let classes = sseq.classes_in_bidegree(...new_bidegree);
        if(classes.length == 0){
            return;
        }
        selected_bidegree = new_bidegree;
        let class_highlighter = document.querySelector("sseq-class-highlighter");
        let result = class_highlighter.clear();
        class_highlighter.highlight(classes, 0.8);
        
        
        socket_listener.send("interact.select_bidegree", {"bidegree" : selected_bidegree});
        display.update();
    });

    // display.addEventListener("mouseover-class", (e) => {
    //     let [c, ms] = e.detail;
    //     document.querySelector("sseq-class-highlighter").fire(c);
        
    // });

    socket_listener.start();
}
