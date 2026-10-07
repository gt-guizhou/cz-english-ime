//! 疯狂听抄输入法 addon 工厂。
#include "qingjian.h"
namespace fcitx {
class CZEnglishIMEFactory final : public AddonFactory {
public:
    AddonInstance *create(AddonManager *manager) override { return new CZEnglishIMEEngine(manager); }
};
}
FCITX_ADDON_FACTORY_V2(qingjian, fcitx::CZEnglishIMEFactory)
