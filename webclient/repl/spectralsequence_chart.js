var Module=typeof pyodide._module!=="undefined"?pyodide._module:{};
Module.checkABI(1);
if(!Module.expectedDataFileDownloads){
    Module.expectedDataFileDownloads=0;
    Module.finishedDataFileDownloads=0;
}
Module.expectedDataFileDownloads++;
(function(){
    var loadPackage = function(metadata){
        var PACKAGE_PATH;
        if(typeof window==="object"){
            PACKAGE_PATH=window["encodeURIComponent"](window.location.pathname.toString().substring(0,window.location.pathname.toString().lastIndexOf("/"))+"/")
        } else if(typeof location!=="undefined"){
            PACKAGE_PATH=encodeURIComponent(location.pathname.toString().substring(0,location.pathname.toString().lastIndexOf("/"))+"/")
        } else {
            throw "using preloaded data can only be done on a web page or in a web worker"
        }
        var PACKAGE_NAME="spectralsequence_chart.data";
        var REMOTE_PACKAGE_BASE="spectralsequence_chart.data";
        if(typeof Module["locateFilePackage"]==="function"&&!Module["locateFile"]){
            Module["locateFile"]=Module["locateFilePackage"];
            err("warning: you defined Module.locateFilePackage, that has been renamed to Module.locateFile (using your locateFilePackage for now)")
        }
        var REMOTE_PACKAGE_NAME=Module["locateFile"]?Module["locateFile"](REMOTE_PACKAGE_BASE,""):REMOTE_PACKAGE_BASE
        ;var REMOTE_PACKAGE_SIZE=metadata.remote_package_size;
        var PACKAGE_UUID=metadata.package_uuid;
        function fetchRemotePackage(packageName,packageSize,callback,errback){
            var xhr=new XMLHttpRequest;xhr.open("GET",packageName,true);
            xhr.responseType="arraybuffer";
            xhr.onprogress=function(event){
                var url=packageName;
                var size=packageSize;
                if(event.total)
                    size=event.total;
                if(event.loaded){
                    if(!xhr.addedTotal){
                        xhr.addedTotal=true;
                        if(!Module.dataFileDownloads)
                            Module.dataFileDownloads={};
                        Module.dataFileDownloads[url]={loaded:event.loaded,total:size}
                    } else {
                        Module.dataFileDownloads[url].loaded=event.loaded
                    }
                    var total=0;
                    var loaded=0;
                    var num=0;
                    for(var download in Module.dataFileDownloads){
                        var data=Module.dataFileDownloads[download];
                        total+=data.total;loaded+=data.loaded;num++
                    }
                    total=Math.ceil(total*Module.expectedDataFileDownloads/num);
                    if(Module["setStatus"]){
                        Module["setStatus"]("Downloading data... ("+loaded+"/"+total+")")
                    }
                } else if(!Module.dataFileDownloads){
                    if(Module["setStatus"])
                    Module["setStatus"]("Downloading data...")
                }
            };
            xhr.onerror=function(event){
                throw new Error("NetworkError for: "+packageName)
            };
            xhr.onload=function(event){
                if(xhr.status==200||xhr.status==304||xhr.status==206||xhr.status==0&&xhr.response){
                    var packageData=xhr.response;callback(packageData)
                }else{
                    throw new Error(xhr.statusText+" : "+xhr.responseURL)
                }
            };
            xhr.send(null)
        }
        function handleError(error){
            console.error("package error:",error)
        }
        var fetchedCallback=null;
        var fetched=Module["getPreloadedPackage"]?Module["getPreloadedPackage"](REMOTE_PACKAGE_NAME,REMOTE_PACKAGE_SIZE):null;
        if(!fetched){
            fetchRemotePackage(REMOTE_PACKAGE_NAME,REMOTE_PACKAGE_SIZE,function(data){
                    if(fetchedCallback){
                        fetchedCallback(data);
                        fetchedCallback=null
                    } else {
                        fetched=data
                    }
                }
                ,handleError
            );
        }
        function runWithFS(){
            function assert(check,msg){
                if(!check)throw msg+(new Error).stack
            }
            Module["FS_createPath"]("/","lib",true,true);
            Module["FS_createPath"]("/lib","python3.8",true,true);
            Module["FS_createPath"]("/lib/python3.8","site-packages",true,true);
            Module["FS_createPath"]("/lib/python3.8/site-packages","spectralsequence_chart",true,true);
            Module["FS_createPath"]("/lib/python3.8/site-packages","spectralsequence_chart-0.0.18-py3.8.egg-info",true,true);
            function DataRequest(start,end,audio){this.start=start;this.end=end;this.audio=audio}
            DataRequest.prototype={
                requests:{},
                open:function(mode,name){this.name=name;this.requests[name]=this;Module["addRunDependency"]("fp "+this.name)},
                send:function(){},
                onload:function(){var byteArray=this.byteArray.subarray(this.start,this.end);this.finish(byteArray)},
                finish:function(byteArray){
                    var that=this;
                    Module["FS_createPreloadedFile"](this.name,null,byteArray,true,true,function(){
                            Module["removeRunDependency"]("fp "+that.name)
                        },
                        function(){
                            if(that.audio){
                                Module["removeRunDependency"]("fp "+that.name)
                            }else{
                                err("Preloading file "+that.name+" failed")
                            }
                        },false,true
                    );
                    this.requests[this.name]=null
                }
            };
            function processPackageData(arrayBuffer){
                Module.finishedDataFileDownloads++;
                assert(arrayBuffer,"Loading data file failed.");
                assert(arrayBuffer instanceof ArrayBuffer,"bad input to processPackageData");
                var byteArray=new Uint8Array(arrayBuffer);
                var curr;
                var compressedData={data:null,cachedOffset:44622,cachedIndexes:[-1,-1],cachedChunks:[null,null],offsets:[0,1205,2251,3174,4223,5318,6169,7228,7918,8951,9994,10709,11409,12293,13336,14248,15353,16442,17490,18513,19294,20468,21558,22184,23076,24196,25136,26223,27347,28444,29239,30138,31154,32006,33067,34409,35706,36564,37859,38966,40162,41274,42249,43495],sizes:[1205,1046,923,1049,1095,851,1059,690,1033,1043,715,700,884,1043,912,1105,1089,1048,1023,781,1174,1090,626,892,1120,940,1087,1124,1097,795,899,1016,852,1061,1342,1297,858,1295,1107,1196,1112,975,1246,1127],successes:[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]};
                compressedData.data=byteArray;assert(typeof Module.LZ4==="object","LZ4 not present - was your app build with  -s LZ4=1  ?");
                Module.LZ4.loadPackage({metadata:metadata,compressedData:compressedData});
                Module["removeRunDependency"]("datafile_spectralsequence_chart.data")
            }
            Module["addRunDependency"]("datafile_spectralsequence_chart.data");
            if(!Module.preloadResults)
                Module.preloadResults={};
            Module.preloadResults[PACKAGE_NAME]={fromCache:false};
            if(fetched){
                processPackageData(fetched);
                fetched=null;
            } else {
                fetchedCallback=processPackageData;
            }
        }
                        
        if(Module["calledRun"]){
            runWithFS()
        } else {
            if(!Module["preRun"])
                Module["preRun"]=[];
            Module["preRun"].push(runWithFS);
        }
    };
    loadPackage({
        files:
            [
                {start:0,audio:0,end:31573,filename:"/lib/python3.8/site-packages/spectralsequence_chart/chart.py"},
                {start:31573,audio:0,end:49649,filename:"/lib/python3.8/site-packages/spectralsequence_chart/chart_class.py"},
                {start:49649,audio:0,end:70094,filename:"/lib/python3.8/site-packages/spectralsequence_chart/chart_edge.py"},
                {start:70094,audio:0,end:76192,filename:"/lib/python3.8/site-packages/spectralsequence_chart/display_primitives.py"},
                {start:76192,audio:0,end:76278,filename:"/lib/python3.8/site-packages/spectralsequence_chart/infinity.py"},
                {start:76278,audio:0,end:80820,filename:"/lib/python3.8/site-packages/spectralsequence_chart/page_property.py"},
                {start:80820,audio:0,end:82951,filename:"/lib/python3.8/site-packages/spectralsequence_chart/serialization.py"},
                {start:82951,audio:0,end:86715,filename:"/lib/python3.8/site-packages/spectralsequence_chart/signal_dict.py"},
                {start:86715,audio:0,end:87321,filename:"/lib/python3.8/site-packages/spectralsequence_chart/utils.py"},
                {start:87321,audio:0,end:87914,filename:"/lib/python3.8/site-packages/spectralsequence_chart/__init__.py"},
                {start:87914,audio:0,end:87915,filename:"/lib/python3.8/site-packages/spectralsequence_chart-0.0.18-py3.8.egg-info/dependency_links.txt"},
                {start:87915,audio:0,end:89384,filename:"/lib/python3.8/site-packages/spectralsequence_chart-0.0.18-py3.8.egg-info/PKG-INFO"},
                {start:89384,audio:0,end:89968,filename:"/lib/python3.8/site-packages/spectralsequence_chart-0.0.18-py3.8.egg-info/SOURCES.txt"},
                {start:89968,audio:0,end:89991,filename:"/lib/python3.8/site-packages/spectralsequence_chart-0.0.18-py3.8.egg-info/top_level.txt"}
            ],
        remote_package_size:48718,
        package_uuid:"176bd1ae-1719-4fcd-a396-77167696c867"
    })
})();