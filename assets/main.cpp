// g++ main.cpp exif.cpp -o app -lraylib -lGLESv2 -lEGL -ldrm -lgbm -lpthread -ldl -lrt -lm 

#include "raylib.h"
#include "exif.h"
#include <fstream>
#include <unistd.h>
#include <algorithm>
#include <vector>
#include <string>
#include <cmath>
#include <thread>
#include <mutex>
#include <deque>
#include <atomic>
#include <iostream>

const int SCREEN_W = 1024;
const int SCREEN_H = 768;

struct PreparedImage {
    Image img;
    std::string date;
    int index;
};

std::deque<PreparedImage> imageQueue;
std::mutex queueMutex;
std::atomic<bool> keepRunning(true);
std::atomic<bool> workerDone(false);

PreparedImage ProcessImageFast(const std::string& fileName, int idx) {
    PreparedImage result = { { 0 }, "", idx };
    std::ifstream file(fileName, std::ios::binary | std::ios::ate);
    if (!file) return result;
    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);
    std::vector<unsigned char> buffer(size);
    if (!file.read((char*)buffer.data(), size)) return result;

    easyexif::EXIFInfo info;
    int orientation = 1;
    if (info.parseFrom(buffer.data(), size) == 0) {
        orientation = info.Orientation;
        if (!info.DateTime.empty()) result.date = info.DateTime;
    }

    const char* ext = GetFileExtension(fileName.c_str());
    result.img = LoadImageFromMemory(ext, buffer.data(), size);
    if (result.img.data != nullptr) {
        if (orientation == 3) { ImageRotateCW(&result.img); ImageRotateCW(&result.img); }
        else if (orientation == 6) { ImageRotateCW(&result.img); }
        else if (orientation == 8) { ImageRotateCCW(&result.img); }

        float scale = std::min((float)SCREEN_W / result.img.width, (float)SCREEN_H / result.img.height);
        if (scale < 1.0f) {
            int targetW = (int)(result.img.width * scale);
            int targetH = (int)(result.img.height * scale);
            ImageResize(&result.img, targetW, targetH);
            
            Rectangle cropRec = { 1, 1, (float)targetW - 2, (float)targetH - 2 };
            ImageCrop(&result.img, cropRec);
        }
    }
    return result;
}

void ImageWorker(std::vector<std::string> imagePaths, int maxImages) {
    int nextIdx = 0;
    while (keepRunning && nextIdx < maxImages) {
        bool shouldLoad = false;
        {
            std::lock_guard<std::mutex> lock(queueMutex);
            shouldLoad = (imageQueue.size() < 3);
        }
        if (shouldLoad) {
            std::string path = imagePaths[nextIdx % imagePaths.size()]; 
            PreparedImage pImg = ProcessImageFast(path, nextIdx);
            {
                std::lock_guard<std::mutex> lock(queueMutex);
                imageQueue.push_back(pImg);
            }
            nextIdx++;
        } else {
            std::this_thread::sleep_for(std::chrono::milliseconds(200));
        }
    }
    workerDone = true;
}

int main(int argc, char* argv[]) {
    float displayTime = 20.0f;
    float fullTime = 5400.0f;
    int opt;
    while ((opt = getopt(argc, argv, "t:f:p")) != -1) {
        switch (opt) {
            case 't': displayTime = std::stof(optarg); break;
            case 'f': fullTime = std::stof(optarg); break;
        }
    }

    std::vector<std::string> images;
    for (int i = optind; i < argc; i++) images.push_back(argv[i]);
    if (images.empty()) return 1;

    int totalLoops = static_cast<int>(ceilf(fullTime / (static_cast<float>(images.size()) * displayTime)));
    int totalImagesToProcess = images.size() * totalLoops;

    std::thread worker(ImageWorker, images, totalImagesToProcess);

    SetConfigFlags(FLAG_WINDOW_UNDECORATED | FLAG_FULLSCREEN_MODE | FLAG_VSYNC_HINT);
    InitWindow(SCREEN_W, SCREEN_H, "Simple Photo Frame");
    SetTargetFPS(30);

    Font myFont = LoadFont("/home/kuba/simple_image_frame/digital-7.ttf");
    Texture2D currentTexture = { 0 };
    Texture2D oldTexture = { 0 };
    std::string currentExifText = "";
    
    float timer = displayTime; 
    float transitionAlpha = 1.0f;
    bool isTransitioning = false;
    float fadeSpeed = 2.0f;

    auto DrawTex = [&](Texture2D tex, float alpha) {
        if (tex.id <= 0) return;
        
        float s = std::min(((float)SCREEN_W - 4.0f)/tex.width, ((float)SCREEN_H - 4.0f)/tex.height);
        
        Rectangle src = { 0, 0, (float)tex.width, (float)tex.height };
        
        Rectangle dest = {
            std::round((SCREEN_W - tex.width * s) / 2.0f),
            std::round((SCREEN_H - tex.height * s) / 2.0f),
            std::round(tex.width * s),
            std::round(tex.height * s)
        };
        
        DrawTexturePro(tex, src, dest, {0, 0}, 0.0f, Fade(WHITE, alpha));
    };

    while (!WindowShouldClose()) {
        float dt = GetFrameTime();
        if (dt > 0.1f) dt = 0.1f; 

        if (isTransitioning) {
            transitionAlpha += dt * fadeSpeed;
            if (transitionAlpha >= 1.0f) {
                transitionAlpha = 1.0f;
                isTransitioning = false;
                timer = 0.0f;
            }
        } else {
            timer += dt;
        }

        bool shouldExit = false;
        PreparedImage nextData;
        bool hasNext = false;

        if ((timer >= displayTime && !isTransitioning) || workerDone) {
            std::lock_guard<std::mutex> lock(queueMutex);
            if (workerDone && imageQueue.empty() && !isTransitioning && timer >= displayTime) {
                shouldExit = true;
            } 
            else if (timer >= displayTime && !isTransitioning && !imageQueue.empty()) {
                nextData = imageQueue.front();
                imageQueue.pop_front();
                hasNext = true;
            }
        }

        if (shouldExit) break;

        if (hasNext) {
            if (oldTexture.id > 0) UnloadTexture(oldTexture);
            oldTexture = currentTexture;
            currentTexture = LoadTextureFromImage(nextData.img);
            SetTextureWrap(currentTexture, TEXTURE_WRAP_CLAMP); 
            SetTextureFilter(currentTexture, TEXTURE_FILTER_BILINEAR); 
            UnloadImage(nextData.img);
            currentExifText = nextData.date;
            transitionAlpha = 0.0f;
            isTransitioning = true;
        }

        BeginDrawing();
            ClearBackground(BLACK);

            if (isTransitioning) {
                DrawTex(oldTexture, 1.0f - transitionAlpha);
                DrawTex(currentTexture, transitionAlpha);
            } else {
                DrawTex(currentTexture, 1.0f);
            }


            if (!currentExifText.empty()) {
                DrawTextEx(myFont, currentExifText.c_str(), { 22, (float)SCREEN_H-38 }, 20, 2, BLACK);
                DrawTextEx(myFont, currentExifText.c_str(), { 20, (float)SCREEN_H-40 }, 20, 2, RAYWHITE);
            }

        EndDrawing();
    }

    keepRunning = false;
    worker.join();
    if (currentTexture.id > 0) UnloadTexture(currentTexture);
    if (oldTexture.id > 0) UnloadTexture(oldTexture);
    CloseWindow();
    return 0;
}
